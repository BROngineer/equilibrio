use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tracing::{event, instrument, Level};

#[derive(Clone, Debug)]
struct Endpoint {
    address: SocketAddr,
    healthy: bool,
}

#[derive(Clone, Debug)]
pub struct Checker {
    endpoints: Arc<RwLock<Vec<Endpoint>>>,
}

impl Checker {
    pub fn new(endpoints: Arc<Vec<SocketAddr>>) -> Checker {
        Checker {
            endpoints: Arc::new(RwLock::new(
                endpoints
                    .iter()
                    .map(|&ep| Endpoint {
                        address: ep,
                        healthy: false,
                    })
                    .collect(),
            )),
        }
    }

    pub async fn get_healthy_endpoints(&self) -> Vec<SocketAddr> {
        let endpoints = self.endpoints.read().await;
        endpoints
            .iter()
            .filter(|&ep| ep.healthy)
            .map(|ep| ep.address)
            .collect::<Vec<SocketAddr>>()
    }

    pub fn run(&self) {
        let checker = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                checker.health_check().await;
            }
        });
    }

    #[instrument(
        name = "health_check",
        target = "balancer::health_check",
        level = Level::INFO,
        skip(self),
        fields(endpoint, is_healthy)
    )]
    async fn health_check(&self) {
        let endpoints = self.endpoints.read().await.clone();

        let checked_endpoints = futures::future::join_all(endpoints.iter().map(|ep| async move {
            let is_healthy = Checker::check_endpoint(&ep.address).await;
            event!(
                Level::DEBUG,
                endpoint = ep.address.to_string(),
                is_healthy = is_healthy,
                message = "endpoint checked"
            );
            Endpoint {
                address: ep.address,
                healthy: is_healthy,
            }
        }))
        .await;

        let mut endpoints_lock = self.endpoints.write().await;
        *endpoints_lock = checked_endpoints;
    }

    async fn check_endpoint(endpoint: &SocketAddr) -> bool {
        TcpStream::connect(endpoint).await.is_ok()
    }
}
