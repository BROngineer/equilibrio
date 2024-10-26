use std::net::SocketAddr;
use clap::Parser;
use tokio::io::copy;
use tokio::net::{TcpListener, TcpStream};

use equilibrio::cmd::args::Args;

async fn forward(mut inbound: TcpStream, addr: SocketAddr) -> tokio::io::Result<()> {
    let mut outbound = TcpStream::connect(addr).await?;

    let (mut ri, mut wi) = inbound.split();
    let (mut ro, mut wo) = outbound.split();

    let client_to_server = copy(&mut ri, &mut wo);
    let server_to_client = copy(&mut ro, &mut wi);

    tokio::try_join!(client_to_server, server_to_client)?;

    Ok(())
}

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let args = Args::parse();
    
    let bind_addr: SocketAddr = format!("{}:{}", args.address, args.port)
        .parse().expect("Invalid socket address");
    let proxy_addr: SocketAddr = args.endpoint.parse().expect("Invalid socket address");


    let listener = TcpListener::bind(bind_addr).await?;
    loop {
        let (inbound, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = forward(inbound, proxy_addr).await {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }
}
