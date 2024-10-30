use crate::balancer::health_check::Checker;
use crate::balancer::ip_hash::IpHashBalancer;
use crate::balancer::round_robin::RoundRobinBalancer;
use async_trait::async_trait;
use futures::future::BoxFuture;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tracing::{event, instrument, Level};

pub mod health_check;
mod ip_hash;
mod round_robin;

pub enum Type {
    RoundRobin,
    IpHash,
}

#[async_trait]
pub trait Balancer: Send + Sync {
    fn get_endpoints(&self) -> Arc<Vec<SocketAddr>>;

    fn next_endpoint(&mut self, addr: SocketAddr) -> Option<SocketAddr>;

    // todo: implement update for endpoints so that only healthy one could be chosen
    fn set_healthy_endpoints(&mut self, healthy_endpoints: Vec<SocketAddr>);

    #[instrument(
        name = "balancer",
        target = "balancer",
        level = Level::INFO,
        skip(self,
        forward_fn)
    )]
    async fn run(
        &mut self,
        bind_address: SocketAddr,
        forward_fn: Arc<
            dyn Fn(TcpStream, SocketAddr) -> BoxFuture<'static, tokio::io::Result<()>>
                + Send
                + Sync,
        >,
    ) -> tokio::io::Result<()> {
        let health_checker = Checker::new(self.get_endpoints());
        let listener = TcpListener::bind(bind_address).await?;

        event!(Level::INFO, "starting health checker");
        health_checker.run();

        event!(Level::INFO, "starting tcp listener");
        loop {
            let (inbound, addr) = listener.accept().await?;
            self.set_healthy_endpoints(health_checker.get_healthy_endpoints().await);
            match self.next_endpoint(addr) {
                None => {
                    event!(Level::WARN, "no available endpoints");
                }
                Some(ep) => {
                    event!(Level::DEBUG, next_endpoint = ep.to_string());
                    let fw_fn = forward_fn.clone();
                    tokio::spawn(async move {
                        if let Err(e) = fw_fn(inbound, ep).await {
                            event!(
                                Level::WARN,
                                target = ep.to_string(),
                                error = e.to_string(),
                                message = "failed to forward"
                            );
                        }
                    });
                }
            };
        }
    }
}

pub fn new(balancer_type: Type, endpoints: Vec<SocketAddr>) -> Box<dyn Balancer> {
    match balancer_type {
        Type::RoundRobin => Box::new(RoundRobinBalancer::new(endpoints)),
        Type::IpHash => Box::new(IpHashBalancer::new(endpoints)),
    }
}
