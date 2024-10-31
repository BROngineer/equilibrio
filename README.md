![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/brongineer/equilibrio/tests.yml?branch=main&logo=github&label=tests)
![Codecov](https://img.shields.io/codecov/c/github/brongineer/equilibrio?logo=codecov)
![GitHub License](https://img.shields.io/github/license/brongineer/equilibrio)

# equilibrio

Simple load-balancing pet-project. Developing it just for fun and to get a taste of Rust.

### Load-balancing algorithms

For now `equilibrio` supports only L4 load-balancing and the following load-balancing strategies:

- Round-Robin
- IP Hash

### Usage

```shell
❯ equilibrio --help
Simple load-balancing project

Usage: equilibrio [OPTIONS]

Options:
  -a, --bind-address <BIND_ADDRESS>  Address to listen on [default: 0.0.0.0]
  -p, --bind-port <BIND_PORT>        Port to listen on [default: 9080]
  -t, --type <BALANCER>              Balancing strategy [default: round-robin] [possible values: round-robin, ip-hash]
  -l, --layer <LAYER>                OSI layer [default: layer4] [possible values: layer4]
  -e, --endpoint <ENDPOINT>          Endpoint address to forward to; flag can be used multiple times
      --log-format <LOG_FORMAT>      Log format [default: text] [possible values: text, json]
      --log-level <LOG_LEVEL>        Log level [default: info] [possible values: trace, debug, info, warn, error]
  -h, --help                         Print help
  -V, --version                      Print version

```
