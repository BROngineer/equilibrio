use std::net::SocketAddr;
use tokio::io::copy;
use tokio::net::{TcpListener, TcpStream};

use equilibrio::cmd::args::parse;

async fn forward(inbound: TcpStream, addr: SocketAddr) -> tokio::io::Result<()> {
    let outbound = TcpStream::connect(addr).await?;

    let (mut ri, mut wi) = inbound.into_split();
    let (mut ro, mut wo) = outbound.into_split();

    let client_to_server = copy(&mut ri, &mut wo);
    let server_to_client = copy(&mut ro, &mut wi);

    tokio::try_join!(client_to_server, server_to_client)?;

    Ok(())
}

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let args = parse();

    let listener = TcpListener::bind(args.bind_address).await?;
    loop {
        let (inbound, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = forward(inbound, args.endpoint).await {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }
}
