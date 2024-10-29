use std::net::SocketAddr;
use async_trait::async_trait;
use crate::balancer::Balancer;

struct EndpointsList {
    endpoints: Vec<SocketAddr>,
    cursor: usize
}

impl EndpointsList {
    fn new(endpoints: Vec<SocketAddr>) -> EndpointsList {
        EndpointsList {
            endpoints,
            cursor: 0
        }
    }
    fn get(&mut self) -> Option<SocketAddr> {
        match self.endpoints.is_empty() {
            true => { None }
            false => {
                let endpoint = self.endpoints[self.cursor];
                self.cursor = (self.cursor + 1) % self.endpoints.len();
                Some(endpoint)
            }
        }
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
    fn get_endpoint(&mut self, _addr: SocketAddr) -> Option<SocketAddr> {
        self.endpoints.get()
    }

    fn set_healthy_endpoints(&mut self, healthy_endpoints: Vec<SocketAddr>) {
        
    }
}