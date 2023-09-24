# rust-orderbook-aggregator

A Rust CLI aggregates orderbook from multiple crypto exchanges.

<img width="1106" alt="image" src="https://github.com/kevinyin9/rust-orderbook-aggregator/assets/20009750/979ece21-2ff1-4a5f-b30e-6e5b3c4110c2">

## Basic Logic
1. Connects to multiple exchanges' websocket feeds at the same time.
2. Pulls orderbooks, using these streaming connections, for a given traded pair of currencies (configurable), from each exchange.
3. Merges and sorts the orderbooks to create a combined orderbook.
4. From the combined book, publishes the spread, top ten bids, and top ten asks, as a stream, through a gRPC server.

## Usage
First, start gRPC server:
```
cargo run --release -p orderbook-merger --bin server
```

Start gRPC client with terminal user interface:
```
cargo run --release -p terminal-ui
```
