use tokio::net::TcpStream;
use std::net::SocketAddr;
use tokio::io::copy;
use futures::future::BoxFuture;

pub fn box_forward_fn(inbound: TcpStream, addr: SocketAddr) -> BoxFuture<'static, tokio::io::Result<()>> {
    Box::pin(forward(inbound, addr))
}

async fn forward(inbound: TcpStream, addr: SocketAddr) -> tokio::io::Result<()> {
    let outbound = TcpStream::connect(addr).await?;
    
    let (mut ri, mut wi) = inbound.into_split();
    let (mut ro, mut wo) = outbound.into_split();
    
    let client_to_server = copy(&mut ri, &mut wo);
    let server_to_client = copy(&mut ro, &mut wi);
    
    tokio::try_join!(client_to_server, server_to_client)?;
    Ok(())
}