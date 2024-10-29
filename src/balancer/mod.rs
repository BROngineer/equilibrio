use std::net::SocketAddr;
use async_trait::async_trait;
use tokio::io::copy;
use tokio::net::{TcpListener, TcpStream};
use crate::balancer::health_check::Checker;
use crate::balancer::ip_hash::IpHashBalancer;
use crate::balancer::round_robin::RoundRobinBalancer;

mod round_robin;
mod ip_hash;
mod health_check;

pub enum BalancerType {
    RoundRobin,
    IpHash
}

#[async_trait]
trait Balancer {
    fn get_endpoint(&mut self, addr: SocketAddr) -> Option<SocketAddr>;
    
    // todo: implement update for endpoints so that only healthy one could be chosen
    fn set_healthy_endpoints(&mut self, healthy_endpoints: Vec<SocketAddr>);
}

pub async fn run(
    bind_address: SocketAddr,
    endpoints: Vec<SocketAddr>,
    balancer_type: BalancerType,
) -> tokio::io::Result<()> {
    let health_checker = Checker::new(endpoints.clone());
    let listener = TcpListener::bind(bind_address).await?;
    let mut balancer: Box<dyn Balancer> = match balancer_type {
        BalancerType::RoundRobin => { Box::new(RoundRobinBalancer::new(endpoints)) }
        BalancerType::IpHash => { Box::new(IpHashBalancer::new(endpoints)) }
    };

    // todo: info log message
    health_checker.start();

    // todo: info log message
    loop {
        let (inbound, addr) = listener.accept().await?;
        balancer.set_healthy_endpoints(health_checker.get_healthy_endpoints());
        match balancer.get_endpoint(addr) {
            None => {
                // todo: warn log message
                eprintln!("No available endpoint");
            }
            Some(ep) => {
                println!("health_checker: {:?}", health_checker);
                // todo: info log message
                println!("forwarding to {}", ep);
                tokio::spawn(async move {
                    if let Err(e) = forward(inbound, ep).await {
                        // todo: error log message
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