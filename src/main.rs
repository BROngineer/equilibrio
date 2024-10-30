use equilibrio::cmd::args::parse;
use equilibrio::forwarder::get_forward_fn;
use equilibrio::{balancer, log};

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let args = parse();
    log::setup(args.log_level, args.log_format).expect("Failed to setup logging");

    let mut balancer = balancer::new(args.balancer_type, args.endpoints);
    let forward_fn = get_forward_fn(args.layer);
    balancer.run(args.bind_address, forward_fn).await
}
