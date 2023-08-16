use config::Config;
use std::collections::HashMap;
use tonic::{transport::Server, Status};

#[tokio::main]
async fn main() -> Result<(), E> { // Should i return Result<(), Error>?
    let config = Config::builder()
        // read the setting.toml
        .add_source(config::File::with_name("src/setting"))
        .build()
        .unwrap()
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();

    Server::builder()
    .serve(config["server-ip"].parse()?)
    .await?;
    Ok(())
}