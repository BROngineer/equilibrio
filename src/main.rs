use equilibrio::balancer;
use equilibrio::cmd::args::parse;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let args = parse();
    let mut balancer = balancer::new(args.balancer_type, args.endpoints);
    balancer.run(args.bind_address).await
}
