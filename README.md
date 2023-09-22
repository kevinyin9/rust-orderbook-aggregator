# rust-orderbook-aggregator

A Rust CLI aggregates orderbook from multiple crypto exchanges.

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
