use std::hash::{Hash, Hasher};
use std::net::{IpAddr, SocketAddr};
use crate::balancer::Balancer;

struct EndpointsList {
    endpoints: Vec<SocketAddr>,
}

impl EndpointsList {
    fn new(endpoints: Vec<SocketAddr>) -> Self {
        EndpointsList {
            endpoints,
        }
    }
    
    fn hash_ip(&self, ip: IpAddr) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        ip.hash(&mut hasher);
        hasher.finish()
    }

    fn get(&self, ip: IpAddr) -> Option<SocketAddr> {
        let hash = self.hash_ip(ip);
        let idx = (hash as usize) % self.endpoints.len();
        Some(self.endpoints[idx])
    }
}

pub struct IpHashBalancer {
    endpoints: EndpointsList,
}

impl IpHashBalancer {
    pub fn new(endpoints: Vec<SocketAddr>) -> Self {
        IpHashBalancer { endpoints: EndpointsList::new(endpoints) }
    }
}

impl Balancer for IpHashBalancer {
    fn get_endpoint(&mut self, addr: SocketAddr) -> Option<SocketAddr> {
        self.endpoints.get(addr.ip())
    }
}