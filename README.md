# equilibrio

Simple load-balancing pet-project.

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
  -a, --bind-address <BIND_ADDRESS>  [default: 0.0.0.0]
  -p, --bind-port <BIND_PORT>        [default: 9080]
  -t, --type <BALANCER>              [default: round-robin] [possible values: round-robin, ip-hash]
  -l, --layer <LAYER>                [default: layer4] [possible values: layer4]
  -e, --endpoint <ENDPOINT>
      --log-format <LOG_FORMAT>      [default: text] [possible values: text, json]
      --log-level <LOG_LEVEL>        [default: info] [possible values: trace, debug, info, warn, error]
  -h, --help                         Print help
  -V, --version                      Print version
```

### Load testing

Running `equilibrio` in front of 3 docker containers with nginx:

- Round-Robin

```shell
❯ docker ps
CONTAINER ID   IMAGE     COMMAND                  CREATED        STATUS        PORTS                                   NAMES
97ad08a14639   nginx     "/docker-entrypoint.…"   25 hours ago   Up 25 hours   0.0.0.0:8082->80/tcp, :::8082->80/tcp   youthful_solomon
d52186bc2369   nginx     "/docker-entrypoint.…"   25 hours ago   Up 25 hours   0.0.0.0:8081->80/tcp, :::8081->80/tcp   interesting_davinci
2eaad5979e67   nginx     "/docker-entrypoint.…"   25 hours ago   Up 25 hours   0.0.0.0:8080->80/tcp, :::8080->80/tcp   nervous_booth
❯ equilibrio -e localhost:8080 -e localhost:8081 -e localhost:8082 --type round-robin > /dev/null &
❯ k6 run load_test.js

     scenarios: (100.00%) 1 scenario, 20 max VUs, 1m0s max duration (incl. graceful stop):
              * default: Up to 20 looping VUs for 30s over 3 stages (gracefulRampDown: 30s, gracefulStop: 30s)


     ✓ status is 200

     checks.........................: 100.00% ✓ 410       ✗ 0
     data_received..................: 101 kB  3.4 kB/s
     data_sent......................: 33 kB   1.1 kB/s
     http_req_blocked...............: avg=32.68µs  min=2.76µs   med=7.39µs   max=898.12µs p(90)=12.96µs  p(95)=53.76µs
     http_req_connecting............: avg=16.59µs  min=0s       med=0s       max=619.22µs p(90)=0s       p(95)=0s
   ✓ http_req_duration..............: avg=1.13ms   min=372.93µs med=1.13ms   max=8.57ms   p(90)=1.47ms   p(95)=1.67ms
       { expected_response:true }...: avg=1.13ms   min=372.93µs med=1.13ms   max=8.57ms   p(90)=1.47ms   p(95)=1.67ms
     http_req_failed................: 0.00%   ✓ 0         ✗ 410
     http_req_receiving.............: avg=90.42µs  min=33.23µs  med=90.5µs   max=250.13µs p(90)=118.78µs p(95)=138.74µs
     http_req_sending...............: avg=58.64µs  min=12.16µs  med=31.63µs  max=8.21ms   p(90)=61.72µs  p(95)=98.34µs
     http_req_tls_handshaking.......: avg=0s       min=0s       med=0s       max=0s       p(90)=0s       p(95)=0s
     http_req_waiting...............: avg=987.23µs min=279.39µs med=1.01ms   max=3.55ms   p(90)=1.31ms   p(95)=1.41ms
     http_reqs......................: 410     13.635233/s
     iteration_duration.............: avg=1s       min=1s       med=1s       max=1s       p(90)=1s       p(95)=1s
     iterations.....................: 410     13.635233/s
   ✓ my_trend.......................: avg=1.1363   min=0.372936 med=1.139825 max=8.576139 p(90)=1.472138 p(95)=1.670869
     vus............................: 1       min=1       max=20
     vus_max........................: 20      min=20      max=20
```

- IP Hash

```shell
❯ docker ps
CONTAINER ID   IMAGE     COMMAND                  CREATED        STATUS        PORTS                                   NAMES
97ad08a14639   nginx     "/docker-entrypoint.…"   25 hours ago   Up 25 hours   0.0.0.0:8082->80/tcp, :::8082->80/tcp   youthful_solomon
d52186bc2369   nginx     "/docker-entrypoint.…"   25 hours ago   Up 25 hours   0.0.0.0:8081->80/tcp, :::8081->80/tcp   interesting_davinci
2eaad5979e67   nginx     "/docker-entrypoint.…"   25 hours ago   Up 25 hours   0.0.0.0:8080->80/tcp, :::8080->80/tcp   nervous_booth
❯ equilibrio -e localhost:8080 -e localhost:8081 -e localhost:8082 --type ip-hash > /dev/null &
❯ k6 run load_test.js

     scenarios: (100.00%) 1 scenario, 20 max VUs, 1m0s max duration (incl. graceful stop):
              * default: Up to 20 looping VUs for 30s over 3 stages (gracefulRampDown: 30s, gracefulStop: 30s)


     ✓ status is 200

     checks.........................: 100.00% ✓ 410       ✗ 0
     data_received..................: 101 kB  3.4 kB/s
     data_sent......................: 33 kB   1.1 kB/s
     http_req_blocked...............: avg=30.06µs  min=5.98µs   med=7.76µs   max=608.64µs p(90)=11.15µs  p(95)=25.68µs
     http_req_connecting............: avg=15.59µs  min=0s       med=0s       max=455.68µs p(90)=0s       p(95)=0s
   ✓ http_req_duration..............: avg=1.3ms    min=642.79µs med=1.27ms   max=3.07ms   p(90)=1.49ms   p(95)=1.66ms
       { expected_response:true }...: avg=1.3ms    min=642.79µs med=1.27ms   max=3.07ms   p(90)=1.49ms   p(95)=1.66ms
     http_req_failed................: 0.00%   ✓ 0         ✗ 410
     http_req_receiving.............: avg=100.68µs min=70.8µs   med=95.13µs  max=287.63µs p(90)=120.15µs p(95)=135.64µs
     http_req_sending...............: avg=40.49µs  min=23.74µs  med=32.12µs  max=282.37µs p(90)=57.62µs  p(95)=89.86µs
     http_req_tls_handshaking.......: avg=0s       min=0s       med=0s       max=0s       p(90)=0s       p(95)=0s
     http_req_waiting...............: avg=1.16ms   min=527.14µs med=1.14ms   max=2.66ms   p(90)=1.34ms   p(95)=1.42ms
     http_reqs......................: 410     13.635202/s
     iteration_duration.............: avg=1s       min=1s       med=1s       max=1s       p(90)=1s       p(95)=1s
     iterations.....................: 410     13.635202/s
   ✓ my_trend.......................: avg=1.305592 min=0.642796 med=1.278478 max=3.072682 p(90)=1.495944 p(95)=1.669579
     vus............................: 1       min=1       max=20
     vus_max........................: 20      min=20      max=20
```

Running `k6` with direct connections to nginx:

```shell
❯ k6 run load_test.js

     scenarios: (100.00%) 1 scenario, 20 max VUs, 1m0s max duration (incl. graceful stop):
              * default: Up to 20 looping VUs for 30s over 3 stages (gracefulRampDown: 30s, gracefulStop: 30s)


     ✓ status is 200

     checks.........................: 100.00% ✓ 410       ✗ 0
     data_received..................: 101 kB  3.4 kB/s
     data_sent......................: 33 kB   1.1 kB/s
     http_req_blocked...............: avg=93.86µs  min=3.19µs   med=7.79µs   max=16.96ms  p(90)=14.99µs  p(95)=118.13µs
     http_req_connecting............: avg=15.07µs  min=0s       med=0s       max=791.53µs p(90)=0s       p(95)=0s
   ✓ http_req_duration..............: avg=789.56µs min=257.51µs med=773.49µs max=6.59ms   p(90)=938.5µs  p(95)=1.04ms
       { expected_response:true }...: avg=789.56µs min=257.51µs med=773.49µs max=6.59ms   p(90)=938.5µs  p(95)=1.04ms
     http_req_failed................: 0.00%   ✓ 0         ✗ 410
     http_req_receiving.............: avg=99.86µs  min=27.06µs  med=98.9µs   max=281.92µs p(90)=126.65µs p(95)=136.62µs
     http_req_sending...............: avg=53.22µs  min=12.83µs  med=31.69µs  max=6.17ms   p(90)=53.13µs  p(95)=95.21µs
     http_req_tls_handshaking.......: avg=0s       min=0s       med=0s       max=0s       p(90)=0s       p(95)=0s
     http_req_waiting...............: avg=636.47µs min=176.77µs med=634.39µs max=2.3ms    p(90)=794.06µs p(95)=878.09µs
     http_reqs......................: 410     13.628685/s
     iteration_duration.............: avg=1s       min=1s       med=1s       max=1.01s    p(90)=1s       p(95)=1s
     iterations.....................: 410     13.628685/s
   ✓ my_trend.......................: avg=0.789564 min=0.25751  med=0.773492 max=6.595966 p(90)=0.938506 p(95)=1.047277
     vus............................: 1       min=1       max=20
     vus_max........................: 20      min=20      max=20
```
