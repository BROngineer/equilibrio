use std::net::SocketAddr;
use tokio::io::copy;
use tokio::net::{TcpListener, TcpStream};
use crate::balancer::ip_hash::IpHashBalancer;
use crate::balancer::round_robin::RoundRobinBalancer;

mod round_robin;
mod ip_hash;

pub enum BalancerType {
    RoundRobin,
    IpHash
}

trait Balancer {
    fn get_endpoint(&mut self, addr: SocketAddr) -> Option<SocketAddr>;
}

pub async fn run(
    bind_address: SocketAddr,
    endpoints: Vec<SocketAddr>,
    balancer_type: BalancerType,
) -> tokio::io::Result<()> {
    let listener = TcpListener::bind(bind_address).await?;
    let mut balancer: Box<dyn Balancer> = match balancer_type {
        BalancerType::RoundRobin => { Box::new(RoundRobinBalancer::new(endpoints)) }
        BalancerType::IpHash => { Box::new(IpHashBalancer::new(endpoints)) }
    };
    loop {
        let (inbound, addr) = listener.accept().await?;
        match balancer.get_endpoint(addr) {
            None => {
                eprintln!("No available endpoint");
            }
            Some(ep) => {
                tokio::spawn(async move {
                    if let Err(e) = forward(inbound, ep).await {
                        eprintln!("Error handling connection: {}", e);
                    }
                });
            }
        };
    }
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