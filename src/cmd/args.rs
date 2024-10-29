use std::net::{SocketAddr, ToSocketAddrs};
use clap::Parser;
use serde::Serialize;
use crate::balancer::Type;
use crate::forwarder::ForwarderLayer;

pub struct Config {
    pub bind_address: SocketAddr,
    pub endpoints: Vec<SocketAddr>,
    pub balancer_type: Type,
    pub layer: ForwarderLayer,
}

#[derive(clap::ValueEnum, Default, Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
enum BalancerType {
    #[default]
    RoundRobin,
    IpHash,
}

#[derive(clap::ValueEnum, Default, Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Layer {
    #[default]
    Layer4,
}

#[derive(Parser, Debug)]
#[clap(
    about = "Simple load-balancing project",
    version = "0.1.0",
    author = "BROngineer",
)]
struct Args {
    #[clap(short = 'a', long, default_value = "0.0.0.0")]
    bind_address: String,
    
    #[clap(short = 'p', long, default_value = "9080")]
    bind_port: u16,

    #[clap(short = 'e', long, default_value = "127.0.0.1:8080")]
    endpoint: Vec<String>,

    #[clap(short = 't', long = "type", default_value_t, value_enum)]
    balancer: BalancerType,
    
    #[clap(short = 'l', long = "layer", default_value_t, value_enum)]
    layer: Layer,
}

pub fn parse() -> Config {
    let args = Args::parse();
    
    let bind_address = format!("{}:{}", args.bind_address, args.bind_port).to_socket_addrs()
        .expect("Failed to parse bind address").next().expect("No socket addresses found");
    let endpoints = args.endpoint.iter()
        .map(|e| e.to_socket_addrs().expect("Failed to parse socket address").next().expect("No socket addresses found"))
        .collect::<Vec<_>>();
    let balancer_type = match args.balancer {
        BalancerType::RoundRobin => { Type::RoundRobin }
        BalancerType::IpHash => { Type::IpHash }
    };
    let layer = match args.layer {
        Layer::Layer4 => { ForwarderLayer::Layer4 }
    };
    
    Config {
        bind_address,
        endpoints,
        balancer_type,
        layer,
    }
}