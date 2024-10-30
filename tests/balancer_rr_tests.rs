use equilibrio::balancer;
use equilibrio::balancer::health_check::Checker;
use equilibrio::balancer::Type;
use equilibrio::forwarder::{get_forward_fn, ForwarderLayer};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

async fn test_balancer(
    balancer_type: Type,
    forwarder_layer: ForwarderLayer,
    address: &str,
    endpoint_addresses: Vec<&str>,
) {
    // setup endpoints
    let forward_address: SocketAddr = address.parse().unwrap();
    let endpoints: Vec<SocketAddr> = endpoint_addresses
        .iter()
        .map(|&s| s.parse().unwrap())
        .collect();

    let message = b"hello";

    // start listening on endpoints
    for endpoint in endpoints.clone() {
        let dummy_server = TcpListener::bind(endpoint).await.unwrap();
        tokio::spawn(async move {
            let (mut dummy_conn, _) = dummy_server.accept().await.unwrap();
            let mut buf = vec![0; 1024];
            let n = dummy_conn.read(&mut buf).await.unwrap();
            assert_eq!(&buf[..n], message);
        });
    }

    let (ready_tx, ready_rx) = oneshot::channel::<()>();

    tokio::time::sleep(Duration::from_secs(1)).await;

    // start balancer
    let forward_fn = get_forward_fn(forwarder_layer);
    let mut balancer = balancer::new(balancer_type, endpoints.clone());
    tokio::spawn(async move {
        let _ = ready_tx.send(());
        let _ = balancer.run(forward_address, forward_fn).await;
    });

    let _ = ready_rx.await;

    // Retry mechanism to attempt connection
    let mut attempt = 0;
    let max_attempts = 10;

    loop {
        let result = TcpStream::connect(forward_address).await;
        match result {
            Ok(mut outbound) => {
                outbound.write_all(message).await.unwrap();
                outbound.shutdown().await.unwrap();
                break;
            }
            Err(e) if attempt < max_attempts => {
                attempt += 1;
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Err(e) => panic!("Failed to connect after several attempts: {}", e),
        }
    }
}

#[tokio::test]
async fn test_balancer_rr() {
    let test_cases = vec![
        (
            Type::RoundRobin,
            ForwarderLayer::Layer4,
            "127.0.0.1:32000",
            vec!["127.0.0.1:32001", "127.0.0.1:32002", "127.0.0.1:32003"],
        ),
        (
            Type::RoundRobin,
            ForwarderLayer::Layer4,
            "127.0.0.1:32004",
            vec!["127.0.0.1:30005", "127.0.0.1:30006", "127.0.0.1:30007"],
        ),
    ];

    for case in test_cases {
        test_balancer(case.0, case.1, case.2, case.3).await;
    }
}
