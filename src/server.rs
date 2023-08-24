use config::Config;
use std::pin::Pin;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::{sync::mpsc, sync::oneshot, sync::watch};
use tokio_stream::{wrappers::WatchStream, Stream};
use tonic::{transport::Server, Request, Response, Status};
use rust_orderbook_merger::{
    orderbook::orderbook::OrderBookOnlyLevels,
    exchange::{exchange::Exchange, binance::Binance},
    orderbooksummary::{
        orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer},
        Empty, Summary, Level,
    },
    Symbol
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

    let (tx3, rx3) = mpsc::channel::<oneshot::Sender<watch::Receiver<Result<Summary, Status>>>>(20);
    // drop(rx3);

    // tokio::spawn(async move {
    //     binance_orderbook.start(tx).await.unwrap();
    // });
    // tokio::spawn(async move {
    //     bitstamp_orderbook.start(tx2).await.unwrap();
    // });
    let reply = Summary{
        spread: 6.4,
        bids: vec![Level{exchange: "binance".to_string(),
            price: 3.5,
            amount: 1.0,
        }],
        asks: vec![Level{exchange: "binance".to_string(),
            price: 3.5,
            amount: 1.0,
        }],
    };
    // tokio::spawn(async move {
    //     let merged_orderbook = HashMap::<&str, OrderBookOnlyLevels>::new();
    //     loop {
    //         // select! {
    //             // val = rx.recv() => {
    //             //     tx.send_replace(Ok(reply)).unwrap();
    //             //     println!("{:?}", val);
    //             // }
    //         // }
    //         tx3.send_replace(Ok(reply)).unwrap();
    //     }
    // });
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