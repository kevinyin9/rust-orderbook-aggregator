use config::Config;
use std::collections::HashMap;
use tokio::{sync::mpsc, sync::oneshot, sync::watch};
use tonic::{transport::Server, Request, Response, Status};

pub mod orderbook {
    tonic::include_proto!("orderbook");
}

use orderbook::{
    orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer},
    Summary, Level,
};

#[derive(Default)]
pub struct OrderbookSummary {}

#[tonic::async_trait]
impl OrderbookAggregator for OrderbookSummary {
    type SummaryStream = Pin<Box<dyn Stream<Item = Result<Summary, Status>> + Send>>;
    async fn book_summary(
        &self,
        request: Request<Summary>,
    ) -> Result<tonic::Response<Self::SummaryStream>, Status> { // Return an instance of type HelloReply
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

        Ok(tonic::Response::new(
            Box::pin(stream) as Self::SummaryStream
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> { // Should i return Result<(), Error>?
    let config = Config::builder()
        // read the setting.toml
        .add_source(config::File::with_name("src/setting"))
        .build()
        .unwrap()
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();

    let orderbook = OrderbookSummary::deafult();

    Server::builder()
    .add_service(OrderbookAggregatorServer::new(orderbook))
    .serve(config["server-ip"].parse()?)
    .await?;

    Ok(())
}