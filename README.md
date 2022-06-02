# exchange-orderbook-merge

This repo is to scratch exchange rate from popular Crypto exchanges and merge them to get summary.
This project has server and client to adopt it into microservice.
Inter-service communication is using gRPC with tonic server.
Currently support Bitstamp, Binance, Kraken, Coinbase Exchanges.

```
USAGE:
    orderbook-server [OPTIONS]

OPTIONS:
    -h, --help               Print help
        --no-binance         Ignore Binance in gRPC stream. Default: false
        --no-bitstamp        Ignore Bitstamp in gRPC stream. Default: false
        --no-kraken          Ignore Kraken in gRPC stream. Default: false
        --no-coinbase        Ignore Coinbase in gRPC stream. Default: false
    -p, --port <PORT>        Port number on which the the gRPC server will be hosted.
                             Default: 50051
    -s, --symbol <SYMBOL>    Currency pair to subscribe to. Default: ETH/BTC
```

Run orderbook server:

```
cargo run --bin orderbook-server
```
or with logs and options:
```
env RUST_LOG=info cargo run --bin orderbook-server -- --symbol ETH/BTC --port 50051
```
Exclude certain exchanges:

```
cargo run --bin orderbook-server -- --no-binance --no-bitstamp
```

Client
-----

Connects to the orderbook server and get the orderbook summary through stream.


```
USAGE:
    orderbook-client [OPTIONS]

OPTIONS:
    -h, --help           Print help
    -p, --port <PORT>    Port number of the orderbook server. Default: 50051
```

Run orderbook client:

```
cargo run --bin orderbook-client
```
or with logs and options:

```
env RUST_LOG=info cargo run --bin orderbook-client -- --port 50051
```

