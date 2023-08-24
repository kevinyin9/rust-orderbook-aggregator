use anyhow::Result;
use config::Config;
use std::collections::HashMap;
use tokio_stream::StreamExt;
use tonic::{Request};

use rust_orderbook_merger::orderbooksummary::{orderbook_aggregator_client::OrderbookAggregatorClient, Empty};
use tonic::transport::Channel;

async fn get_orderbook_summary(mut client: OrderbookAggregatorClient<Channel>) -> Result<()> {
    let request = Request::new(Empty{});
    
    let mut stream = client.book_summary(request).await?.into_inner();

    // while let Some(message) = stream.message().await? {
    //     println!("{:?}", message);
    // }
    while let Some(result) = stream.next().await {
        match result {
            Ok(summary) => println!("\n{:#?}", summary),
            Err(err) => {
                return Err(err.into());
            }
        };
    }
    println!("stream closed");
    Ok(())
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

    // client uses https://IP:Port
    let address = format!("https://{}:{}", config.get("server-ip").unwrap(), config.get("server-port").unwrap());

    println!("{:?}", address);

    let client = OrderbookAggregatorClient::connect(address).await?;

    get_orderbook_summary(client).await?;

    Ok(())
}