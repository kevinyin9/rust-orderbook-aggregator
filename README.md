# rust-orderbook-merger

1. Connects to two exchanges' websocket feeds at the same time,
2. Pulls order books, using these streaming connections, for a given traded pair of currencies (configurable), from each exchange,
3. Merges and sorts the order books to create a combined order book,
4. From the combined book, publishes the spread, top ten bids, and top ten asks, as a stream, through a gRPC server.