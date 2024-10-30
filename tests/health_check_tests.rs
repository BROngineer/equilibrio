use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;

use equilibrio::balancer::health_check::Checker;

async fn test_health_check(endpoint_addresses: Vec<&str>) {
    // setup endpoints
    let endpoints: Vec<SocketAddr> = endpoint_addresses
        .iter()
        .map(|&s| s.parse().unwrap())
        .collect();

    for endpoint in endpoints.clone() {
        let listener = TcpListener::bind(endpoint).await.unwrap();
        tokio::spawn(async move {
            listener.accept().await.unwrap();
        });
    }

    let health_checker = Checker::new(Arc::new(endpoints.clone()));
    health_checker.run();

    tokio::time::sleep(Duration::from_secs(1)).await;
    let healthy_endpoints = health_checker.get_healthy_endpoints().await;
    assert_eq!(endpoints, healthy_endpoints);
}

#[tokio::test]
async fn test_health_checker() {
    let test_cases = vec![
        vec!["127.0.0.1:30001", "127.0.0.1:30002", "127.0.0.1:30003"],
        vec!["127.0.0.1:30004", "127.0.0.1:30005", "127.0.0.1:30006"],
    ];

    for case in test_cases {
        test_health_check(case.to_owned()).await;
    }
}
