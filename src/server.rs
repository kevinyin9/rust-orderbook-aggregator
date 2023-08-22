use config::Config;
use std::pin::Pin;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::{sync::mpsc, sync::oneshot, sync::watch};
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tonic::{transport::Server, Request, Response, Status};

pub mod orderbook {
    tonic::include_proto!("orderbook");
}

use orderbook::{
    orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer},
    Empty, Summary, Level,
};

#[derive(Debug)]
pub struct OrderbookSummary {
    summary: mpsc::Sender::<oneshot::Sender<watch::Receiver<Result<Summary, Status>>>>
}

#[tonic::async_trait]
impl OrderbookAggregator for OrderbookSummary {
    // type BookSummaryStream = Pin<Box<dyn Stream<Item = Result<Summary, Status>> + Send>>;
    type BookSummaryStream = ReceiverStream<Result<Summary, Status>>;
    async fn book_summary(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        println!("Got a request");
        
        let reply = orderbook::Summary{
            spread: 6.4,
            bids: vec![orderbook::Level{exchange: "binance".to_string(),
                price: 3.5,
                amount: 1.0,
            }],
            asks: vec![orderbook::Level{exchange: "binance".to_string(),
                price: 3.5,
                amount: 1.0,
            }],
        };

        let (mut tx, rx) = mpsc::channel(4);

        tx.send(Ok(reply)).await.unwrap();

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

async fn start(symbol: Symbol) {
    
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

    let (tx2, mut rx2) =
        mpsc::channel::<oneshot::Sender<watch::Receiver<Result<Summary, Status>>>>(100);

    let orderbook_summary = OrderbookSummary {
        summary: tx2,
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