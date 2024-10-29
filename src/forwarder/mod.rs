use futures::future::BoxFuture;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;

mod l4_forwarder;

pub enum ForwarderLayer {
    Layer4,
}

pub fn get_forward_fn(
    layer: ForwarderLayer,
) -> Arc<dyn Fn(TcpStream, SocketAddr) -> BoxFuture<'static, tokio::io::Result<()>> + Send + Sync> {
    match layer {
        ForwarderLayer::Layer4 => create_forward_fn(l4_forwarder::box_forward_fn),
    }
}

fn create_forward_fn<F>(
    forward_fn: F,
) -> Arc<dyn Fn(TcpStream, SocketAddr) -> BoxFuture<'static, tokio::io::Result<()>> + Send + Sync>
where
    F: Fn(TcpStream, SocketAddr) -> BoxFuture<'static, tokio::io::Result<()>>
        + Send
        + Sync
        + 'static,
{
    Arc::new(move |stream, addr| forward_fn(stream, addr))
}
