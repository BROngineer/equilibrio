use futures::future::BoxFuture;
use std::net::SocketAddr;
use tokio::io::copy;
use tokio::net::TcpStream;
use tracing::{event, instrument, Level};

pub fn box_forward_fn(
    inbound: TcpStream,
    addr: SocketAddr,
) -> BoxFuture<'static, tokio::io::Result<()>> {
    Box::pin(forward(inbound, addr))
}

#[instrument(
    name = "forwarder",
    target = "balancer::forward",
    level = Level::INFO,
    skip(inbound, addr),
    fields(source, target)
)]
async fn forward(inbound: TcpStream, addr: SocketAddr) -> tokio::io::Result<()> {
    let outbound = TcpStream::connect(addr).await?;

    let (mut ri, mut wi) = inbound.into_split();
    let (mut ro, mut wo) = outbound.into_split();

    tracing::Span::current()
        .record("source", wo.local_addr()?.to_string())
        .record("target", addr.to_string());

    let client_to_server = copy(&mut ri, &mut wo);
    let server_to_client = copy(&mut ro, &mut wi);

    event!(Level::INFO, "forwarding");
    tokio::try_join!(client_to_server, server_to_client)?;

    Ok(())
}
