use std::net::SocketAddr;
use async_trait::async_trait;
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

#[async_trait]
trait Balancer: Send + Sync {
    fn get_endpoint(&mut self, addr: SocketAddr) -> Option<SocketAddr>;
    // async fn check_health(&mut self) -> Vec<SocketAddr>;
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
    
    // let mut endpoints: Vec<SocketAddr> = Vec::new();
    // tokio::spawn(async move {
    //     loop {
    //         endpoints = balancer.check_health().await;
    //         tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    //     }
    // });
    
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