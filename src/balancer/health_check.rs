use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::TcpStream;

#[derive(Clone, Debug)]
struct Endpoint {
    address: SocketAddr,
    healthy: bool,
}

#[derive(Clone, Debug)]
pub struct Checker {
    endpoints: Arc<Mutex<Vec<Endpoint>>>,
}

impl Checker {
    pub fn new(endpoints: Vec<SocketAddr>) -> Checker {
        Checker {
            endpoints: Arc::new(Mutex::new(endpoints.iter().map(|&ep| { Endpoint { address: ep, healthy: false } }).collect())),
        }
    }
    
    pub fn get_healthy_endpoints(&self) -> Vec<SocketAddr> {
        self.endpoints.lock().unwrap().iter()
            .filter(|&ep| { ep.healthy })
            .map(|ep| ep.address)
            .collect::<Vec<SocketAddr>>()
    }
    
    pub fn start(&self) {
        let mut checker = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                checker.health_check().await;
                interval.tick().await;
            }
        });
    }
    
    async fn health_check(&mut self) {
        let endpoints = {
            let endpoints = self.endpoints.lock().unwrap();
            endpoints.clone()
        };
        
        let mut checked_endpoints: Vec<Endpoint> = Vec::with_capacity(endpoints.len());
        for ep in endpoints.iter() {
            let is_healthy = Checker::check_endpoint(&ep.address).await;
            println!("endpoint {} is healthy: {}", ep.address, is_healthy);
            checked_endpoints.push(Endpoint { address: ep.address, healthy: is_healthy });
        }
        
        let mut endpoints_lock = self.endpoints.lock().unwrap();
        *endpoints_lock = checked_endpoints;
    }
    
    async fn check_endpoint(endpoint: &SocketAddr) -> bool {
        TcpStream::connect(endpoint).await.is_ok()
    }
}