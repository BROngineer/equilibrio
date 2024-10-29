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
pub trait Balancer: Send + Sync {
    fn get_endpoints(&self) -> Vec<SocketAddr>;
    
    fn next_endpoint(&mut self, addr: SocketAddr) -> Option<SocketAddr>;
    
    // todo: implement update for endpoints so that only healthy one could be chosen
    fn set_healthy_endpoints(&mut self, healthy_endpoints: Vec<SocketAddr>);
    
    async fn run(&mut self, bind_address: SocketAddr) -> tokio::io::Result<()> {
        let health_checker = Checker::new(self.get_endpoints());
        let listener = TcpListener::bind(bind_address).await?;

        // todo: info log message
        health_checker.run();

        // todo: info log message
        loop {
            let (inbound, addr) = listener.accept().await?;
            self.set_healthy_endpoints(health_checker.get_healthy_endpoints());
            match self.next_endpoint(addr) {
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
}

pub fn new(
    balancer_type: BalancerType,
    endpoints: Vec<SocketAddr>,
) -> Box<dyn Balancer> {
    match balancer_type {
        BalancerType::RoundRobin => Box::new(RoundRobinBalancer::new(endpoints)),
        BalancerType::IpHash => Box::new(IpHashBalancer::new(endpoints)),
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