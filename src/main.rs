use equilibrio::balancer;
use equilibrio::cmd::args::parse;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let args = parse();
    balancer::run(args.bind_address, args.endpoints, args.balancer_type).await
}
