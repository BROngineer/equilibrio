use crate::balancer::Balancer;
use async_trait::async_trait;
use std::hash::{Hash, Hasher};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

struct EndpointsList {
    endpoints: Arc<Vec<SocketAddr>>,
}

impl EndpointsList {
    fn new(endpoints: Vec<SocketAddr>) -> Self {
        EndpointsList {
            endpoints: Arc::new(endpoints),
        }
    }

    fn hash_ip(&self, ip: IpAddr) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        ip.hash(&mut hasher);
        hasher.finish()
    }

    fn next(&self, ip: IpAddr) -> Option<SocketAddr> {
        let hash = self.hash_ip(ip);
        let idx = (hash as usize) % self.endpoints.len();
        Some(self.endpoints[idx])
    }

    fn endpoints(&self) -> Arc<Vec<SocketAddr>> {
        Arc::clone(&self.endpoints)
    }
}

pub struct IpHashBalancer {
    endpoints: EndpointsList,
}

impl IpHashBalancer {
    pub fn new(endpoints: Vec<SocketAddr>) -> Self {
        IpHashBalancer {
            endpoints: EndpointsList::new(endpoints),
        }
    }
}

#[async_trait]
impl Balancer for IpHashBalancer {
    fn get_endpoints(&self) -> Arc<Vec<SocketAddr>> {
        self.endpoints.endpoints()
    }

    fn next_endpoint(&mut self, addr: SocketAddr) -> Option<SocketAddr> {
        self.endpoints.next(addr.ip())
    }

    fn set_healthy_endpoints(&mut self, healthy_endpoints: Vec<SocketAddr>) {
        self.endpoints.endpoints = Arc::new(healthy_endpoints.to_vec())
    }
}
