use equilibrio::balancer;
use equilibrio::balancer::Type;
use equilibrio::forwarder::{get_forward_fn, ForwarderLayer};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio::time::Timeout;

const REQUEST: &[u8; 7] = b"REQUEST";
const RESPONSE: &[u8; 8] = b"RESPONSE";

fn setup_mock_servers(endpoints: Vec<SocketAddr>) -> Vec<Timeout<JoinHandle<()>>> {
    let mut mock_server_handles = Vec::<Timeout<JoinHandle<()>>>::new();
    for endpoint in endpoints {
        let mock_server_handle = tokio::time::timeout(
            Duration::from_secs(1),
            tokio::spawn(async move {
                let mock_listener = TcpListener::bind(endpoint).await.unwrap();
                loop {
                    let (mut socket, _) = mock_listener.accept().await.unwrap();
                    let mut buf = vec![0u8; 1024];
                    let read = socket.read(&mut buf).await;
                    match read {
                        Ok(n) if n < 1 => {}
                        Ok(n) => {
                            assert_eq!(&buf[..n], REQUEST);
                            socket.write_all(RESPONSE).await.unwrap();
                            break;
                        }
                        Err(e) => {
                            panic!("{:?}", e);
                        }
                    }
                }
            }),
        );
        mock_server_handles.push(mock_server_handle);
    }
    mock_server_handles
}

fn setup_mock_client(forward_address: SocketAddr, num_requests: usize) -> Timeout<JoinHandle<()>> {
    tokio::time::timeout(
        Duration::from_secs(1),
        tokio::spawn(async move {
            for _ in 0..num_requests {
                let mut attempt = 0;
                let max_attempts = 10;
                loop {
                    let result = TcpStream::connect(forward_address).await;
                    match result {
                        Ok(mut client_socket) => {
                            client_socket.write_all(REQUEST).await.unwrap();
                            let mut buf = [0; 1024];
                            let read = client_socket.read(&mut buf).await;
                            if let Ok(n) = read {
                                assert_eq!(client_socket.peer_addr().unwrap(), forward_address);
                                assert_eq!(&buf[0..n], RESPONSE);
                                client_socket.shutdown().await.unwrap();
                                break;
                            }
                        }
                        Err(_) if attempt < max_attempts => {
                            attempt += 1;
                            tokio::time::sleep(Duration::from_millis(50)).await;
                        }
                        Err(e) => {
                            panic!("{:?}", e);
                        }
                    }
                }
            }
        }),
    )
}

async fn test_balancer(
    balancer_type: Type,
    forwarder_layer: ForwarderLayer,
    address: &str,
    endpoint_addresses: Vec<&str>,
) {
    let forward_address: SocketAddr = address.parse().unwrap();
    let endpoints: Vec<SocketAddr> = endpoint_addresses
        .iter()
        .map(|&s| s.parse().unwrap())
        .collect();

    // setup and run mocks
    let mut mocks = setup_mock_servers(endpoints.clone());
    let client = setup_mock_client(forward_address, mocks.len());
    mocks.push(client);

    // setup and run balancer
    let mut balancer = balancer::new(balancer_type, endpoints);
    let forward_fn = get_forward_fn(forwarder_layer);
    tokio::spawn(async move { balancer.run(forward_address, forward_fn).await });

    // wait until futures complete
    futures::future::join_all(mocks).await;
}

#[tokio::test]
async fn test_balancer_rr() {
    let test_cases = vec![(
        Type::RoundRobin,
        ForwarderLayer::Layer4,
        "127.0.0.1:32000",
        vec!["127.0.0.1:32001", "127.0.0.1:32002", "127.0.0.1:32003"],
    )];

    for case in test_cases {
        test_balancer(case.0, case.1, case.2, case.3).await;
    }
}

#[tokio::test]
async fn test_balancer_ip_hash() {
    let test_cases = vec![(
        Type::IpHash,
        ForwarderLayer::Layer4,
        "127.0.0.1:32004",
        vec!["127.0.0.1:32005", "127.0.0.1:32006", "127.0.0.1:32007"],
    )];

    for case in test_cases {
        test_balancer(case.0, case.1, case.2, case.3).await;
    }
}
