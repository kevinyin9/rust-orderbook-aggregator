use config::Config;
use std::collections::HashMap;
use tonic::{transport::Server, Status};
use orderbook::{OrderbookAggregator, Summary, Level}

#[tonic::async_trait]
impl OrderbookAggregator for MyOrderbook {
    async fn BookSummary(
        &self
    ) -> Result<()> { // Return an instance of type HelloReply
        println!("Got a request");
        
        Ok() // Send back our formatted greeting
    }
}

#[tokio::main]
async fn main() -> Result<(), E> { // Should i return Result<(), Error>?
    let config = Config::builder()
        // read the setting.toml
        .add_source(config::File::with_name("src/setting"))
        .build()
        .unwrap()
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();

    let orderbook = MyOrderbook::default();

    Server::builder()
    .add_service(GreeterServer::new(orderbook))
    .serve(config["server-ip"].parse()?)
    .await?;
    Ok(())
}