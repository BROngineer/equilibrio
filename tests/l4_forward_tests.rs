use equilibrio::forwarder::{get_forward_fn, ForwarderLayer};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

async fn test_l4_forward(fw_address: &str, tgt_address: &str) {
    // setup forward server
    let forward_address: SocketAddr = fw_address.parse().unwrap();
    let listener = TcpListener::bind(&forward_address).await.unwrap();
    let (tx, rx) = oneshot::channel::<()>();

    // setup target address and message
    let message = b"hello";
    let target_address: SocketAddr = tgt_address.parse().unwrap();

    tokio::spawn(async move {
        let (inbound, _) = listener.accept().await.unwrap();
        get_forward_fn(ForwarderLayer::Layer4)(inbound, target_address)
            .await
            .unwrap();
        tx.send(()).unwrap();
    });

    // setup dummy server to act as forward target
    let dummy_server = TcpListener::bind(target_address).await.unwrap();
    let dummy_server_task = tokio::spawn(async move {
        let (mut dummy_conn, _) = dummy_server.accept().await.unwrap();
        let mut buf = vec![0; 1024];
        let n = dummy_conn.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], message);
    });

    let mut outbound = TcpStream::connect(forward_address).await.unwrap();
    outbound.write_all(message).await.unwrap();
    outbound.shutdown().await.unwrap();

    rx.await.unwrap();

    dummy_server_task.await.unwrap();
}

#[tokio::test]
async fn test_forwarder() {
    let test_cases = vec![
        ("127.0.0.1:31001", "127.0.0.1:31002"),
        ("127.0.0.1:31003", "127.0.0.1:31004"),
        ("127.0.0.1:31005", "127.0.0.1:31006"),
    ];

    for case in test_cases {
        test_l4_forward(case.0, case.1).await;
    }
}
