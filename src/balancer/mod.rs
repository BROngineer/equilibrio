use futures::future::BoxFuture;
use std::net::SocketAddr;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::net::{TcpListener, TcpStream};
use crate::balancer::health_check::Checker;
use crate::balancer::ip_hash::IpHashBalancer;
use crate::balancer::round_robin::RoundRobinBalancer;

mod round_robin;
mod ip_hash;
mod health_check;

pub enum Type {
    RoundRobin,
    IpHash
}

#[async_trait]
pub trait Balancer: Send + Sync {
    fn get_endpoints(&self) -> Arc<Vec<SocketAddr>>;

    fn next_endpoint(&mut self, addr: SocketAddr) -> Option<SocketAddr>;
    
    // todo: implement update for endpoints so that only healthy one could be chosen
    fn set_healthy_endpoints(&mut self, healthy_endpoints: Vec<SocketAddr>);

    async fn run(
        &mut self, 
        bind_address: SocketAddr, 
        forward_fn: Arc<dyn Fn(TcpStream, SocketAddr) -> BoxFuture<'static, tokio::io::Result<()>> + Send + Sync>
    ) -> tokio::io::Result<()> {
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
                    let fw_fn = forward_fn.clone();
                    tokio::spawn(async move {
                        if let Err(e) = fw_fn(inbound, ep).await {
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
    balancer_type: Type,
    endpoints: Vec<SocketAddr>,
) -> Box<dyn Balancer> {
    match balancer_type {
        Type::RoundRobin => Box::new(RoundRobinBalancer::new(endpoints)),
        Type::IpHash => Box::new(IpHashBalancer::new(endpoints)),
    }
}
