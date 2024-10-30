use equilibrio::balancer;
use equilibrio::balancer::Type;
use equilibrio::forwarder::{get_forward_fn, ForwarderLayer};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

async fn test_balancer(
    balancer_type: Type,
    forwarder_layer: ForwarderLayer,
    address: &str,
    endpoint_addresses: Vec<&str>,
) {
    let request_message = b"request";
    let response_message = b"response";
    let forward_address: SocketAddr = address.parse().unwrap();
    let endpoints: Vec<SocketAddr> = endpoint_addresses
        .iter()
        .map(|&s| s.parse().unwrap())
        .collect();

    let mut mock_server_handles = Vec::<JoinHandle<()>>::new();
    for endpoint in endpoints.clone() {
        let mock_server_handle = tokio::spawn(async move {
            let mock_listener = TcpListener::bind(endpoint).await.unwrap();
            loop {
                let (mut socket, _) = mock_listener.accept().await.unwrap();
                let mut buf = vec![0u8; 1024];
                let read = socket.read(&mut buf).await;
                match read {
                    Ok(n) if n < 1 => {}
                    Ok(n) => {
                        assert_eq!(&buf[..n], request_message);
                        socket.write_all(response_message).await.unwrap();
                        break;
                    }
                    Err(e) => {
                        panic!("{:?}", e);
                    }
                }
            }
        });
        mock_server_handles.push(mock_server_handle);
    }

    let mut client_handles = Vec::<JoinHandle<()>>::new();
    for _endpoint in endpoints.clone() {
        let client = tokio::spawn(async move {
            let mut attempt = 0;
            let max_attempts = 10;
            loop {
                let result = TcpStream::connect(forward_address).await;
                match result {
                    Ok(mut client_socket) => {
                        client_socket.write_all(request_message).await.unwrap();
                        let mut buf = [0; 1024];
                        let n = client_socket.read(&mut buf).await.unwrap();
                        assert_eq!(client_socket.peer_addr().unwrap(), forward_address);
                        assert_eq!(&buf[0..n], response_message);
                        client_socket.shutdown().await.unwrap();
                        break;
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
        });
        client_handles.push(client);
    }

    let mut balancer = balancer::new(balancer_type, endpoints);
    let forward_fn = get_forward_fn(forwarder_layer);
    tokio::spawn(async move { balancer.run(forward_address, forward_fn).await });

    futures::future::join_all(mock_server_handles).await;
    futures::future::join_all(client_handles).await;
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
