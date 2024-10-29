use std::net::SocketAddr;
use std::sync::Arc;
use async_trait::async_trait;
use crate::balancer::Balancer;

struct EndpointsList {
    endpoints: Arc<Vec<SocketAddr>>,
    cursor: usize
}

impl EndpointsList {
    fn new(endpoints: Vec<SocketAddr>) -> EndpointsList {
        EndpointsList {
            endpoints: Arc::new(endpoints),
            cursor: 0
        }
    }
    fn next(&mut self) -> Option<SocketAddr> {
        match self.endpoints.is_empty() {
            true => { None }
            false => {
                let endpoint = self.endpoints[self.cursor];
                self.cursor = (self.cursor + 1) % self.endpoints.len();
                Some(endpoint)
            }
        }
    }

    fn endpoints(&self) -> Arc<Vec<SocketAddr>> {
        Arc::clone(&self.endpoints)
    }
}

pub struct RoundRobinBalancer {
    endpoints: EndpointsList,
}

impl RoundRobinBalancer {
    pub fn new(endpoints: Vec<SocketAddr>) -> Self {
        RoundRobinBalancer {endpoints: EndpointsList::new(endpoints)}
    }
}

#[async_trait]
impl Balancer for RoundRobinBalancer {
    fn get_endpoints(&self) -> Arc<Vec<SocketAddr>> {
        self.endpoints.endpoints()
    }
    
    fn next_endpoint(&mut self, _addr: SocketAddr) -> Option<SocketAddr> {
        self.endpoints.next()
    }

    fn set_healthy_endpoints(&mut self, healthy_endpoints: Vec<SocketAddr>) {
        
    }
}