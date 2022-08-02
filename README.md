## Crypto Exchange Data Aggregator

This project consolidates order book data from various cryptocurrency exchanges into a unified view. It's designed as a microservice architecture with a gRPC-based communication layer.


**Server (orderbook-server)**

```
cargo run --bin orderbook-server 
```

**Server Options:**

```
orderbook-server [OPTIONS]

OPTIONS:
    -h, --help               Print help information
    -p, --port <PORT>        Server port (default: 50051)
    -s, --symbol <SYMBOL>    Currency pair (default: ETH/BTC)
    --no-binance             Disable Binance data
    --no-bitstamp            Disable Bitstamp data
    --no-kraken              Disable Kraken data
    --no-coinbase            Disable Coinbase data 
```

**Example:**

```
env RUST_LOG=info cargo run --bin orderbook-server -- --symbol BTC/USD --port 50052 --no-binance
```


**Client (orderbook-client)**

```
cargo run --bin orderbook-client
```

**Client Options:**

```
orderbook-client [OPTIONS]

OPTIONS:
    -h, --help           Print help information
    -p, --port <PORT>    Server port (default: 50051)
```

**Example:**

```
env RUST_LOG=info cargo run --bin orderbook-client -- --port 50052
```


**Supported Exchanges:**

* Binance
* Bitstamp
* Kraken
* Coinbase 
