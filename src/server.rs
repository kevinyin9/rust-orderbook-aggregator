use config::Config;
use std::pin::Pin;
use futures::Stream;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::{sync::mpsc, sync::oneshot, sync::watch, select};
use tokio_stream::wrappers::WatchStream;
use tonic::{transport::Server, Request, Response, Status};
use rust_orderbook_merger::{
    orderbook::orderbook::OrderBookOnlyLevels,
    exchange::{exchange::Exchange, binance::Binance},
    orderbook_summary::{
        orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer},
        Empty, Summary, Level,
    },
    Symbol, ExchangeName,
};

#[derive(Debug)]
pub struct OrderbookSummary {
    summary: mpsc::Sender::<oneshot::Sender<watch::Receiver<Result<Summary, Status>>>>
}

#[tonic::async_trait]
impl OrderbookAggregator for OrderbookSummary {
    // type BookSummaryStream = Pin<Box<dyn Stream<Item = Result<Summary, Status>> + Send>>;
    type BookSummaryStream = WatchStream<Result<Summary, Status>>;
    async fn book_summary(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        println!("Got a request");
        
        let (tx, rx) = oneshot::channel::<watch::Receiver<Result<Summary, Status>>>();
        self.summary.send(tx).await.unwrap();
        let rx_summary = rx.await.unwrap();

        let stream = WatchStream::new(rx_summary);

        // Ok(Response::new(
        //     Box::pin(stream) as Self::WatchSummaryStream
        // ))
        Ok(Response::new(stream))
    }
}

async fn start(symbol: Symbol) -> mpsc::Sender::<oneshot::Sender<watch::Receiver<Result<Summary, Status>>>>{

    let binance_orderbook = Binance::new_exchange(symbol).await.unwrap();
    // let bitstamp_orderbook = Bitstamp::new(symbol).await.unwrap();
    
    let (tx, mut rx) = mpsc::channel::<OrderBookOnlyLevels>(20);
    let tx2 = tx.clone();

    let (tx3, mut rx3) = mpsc::channel::<oneshot::Sender<watch::Receiver<Result<Summary, Status>>>>(20);

    let (tx4, rx4) = watch::channel(Ok(Summary::default()));
    drop(rx4);

    tokio::spawn(async move {
        binance_orderbook.start(tx).await.unwrap();
    });
    // tokio::spawn(async move {
    //     bitstamp_orderbook.start(tx2).await.unwrap();
    // });
    // let reply = Summary{
    //     spread: 6.4,
    //     bids: vec![Level{exchange: "binance".to_string(),
    //         price: 3.5,
    //         amount: 1.0,
    //     }],
    //     asks: vec![Level{exchange: "binance".to_string(),
    //         price: 3.5,
    //         amount: 1.0,
    //     }],
    // };
    tokio::spawn(async move {
        let mut exchange_to_orderbook = HashMap::<ExchangeName, OrderBookOnlyLevels>::new();
        loop {
            select! {
                // Receive a one shot sender that sends a summary receiver back to the server.
                val = rx3.recv() => {
                    if let Some(oneshot_sender) = val {
                        oneshot_sender.send(tx4.subscribe()).unwrap();
                        // tracing::info!("summary_count: {}, rx count: {}", summary_count, tx.receiver_count());
                    }
                },
                // Receive book levels from the order books.
                val = rx.recv() => {
                    if let Some(book_levels) = val {
                        match book_levels {
                            OrderBookOnlyLevels { exchange: ExchangeName::BITSTAMP, .. }  => {
                                exchange_to_orderbook.insert(ExchangeName::BITSTAMP, book_levels);
                            }
                            OrderBookOnlyLevels { exchange: ExchangeName::BINANCE, .. } => {
                                exchange_to_orderbook.insert(ExchangeName::BINANCE, book_levels);
                            }
                        }
                        if exchange_to_orderbook.len() < 2 {
                            continue;
                        }

                        // Book levels are stored in the hashmap above and a new summary created from both exchanges
                        // every time an update is received from either.
                        // let current_levels = exchange_to_orderbook.values().map(|v| v.clone()).collect::<Vec<BookLevels>>();
                        // let summary = make_summary(current_levels, symbol);

                        // tx.send_replace(Ok(summary)).unwrap();
                        // summary_count += 1;
                    }
                },
            }
        }
    });
    tx3
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::builder()
        // read the setting.toml
        .add_source(config::File::with_name("src/setting"))
        .build()
        .unwrap()
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();

    let orderbook_summary = OrderbookSummary {
        summary: start(Symbol::BTCUSDT).await,
    };

    let svc = OrderbookAggregatorServer::new(orderbook_summary);

    // Server uses IP:Port (SocketAddr)
    let address = format!("{}:{}", config.get("server-ip").unwrap(), config.get("server-port").unwrap()).parse()?;

    println!("{:?}", address);

    Server::builder()
        .add_service(svc)
        .serve(address)
        .await?;

    Ok(())
}