use config::Config;
use std::collections::HashMap;
use tokio::{sync::mpsc, sync::oneshot, sync::watch, select};
use tokio_stream::wrappers::WatchStream;
use tonic::{transport::Server, Request, Response, Status};
use orderbook_merger::{
    orderbook::orderbook::OrderBookOnlyLevels,
    exchanges::{exchange::Exchange, binance::Binance, bitstamp::Bitstamp},
    orderbook_summary::{
        orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer},
        Empty, Summary,
    },
    make_summary, Symbol, ExchangeName,
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
    let bitstamp_orderbook = Bitstamp::new_exchange(symbol).await.unwrap();
    
    let (tx, mut rx) = mpsc::channel::<OrderBookOnlyLevels>(20);
    let tx2 = tx.clone();

    let (tx3, mut rx3) = mpsc::channel::<oneshot::Sender<watch::Receiver<Result<Summary, Status>>>>(20);

    let (tx4, rx4) = watch::channel(Ok(Summary::default()));
    drop(rx4);

    // tokio::spawn(async move {
    //     binance_orderbook.start(tx).await.unwrap();
    // });
    tokio::spawn(async move {
        bitstamp_orderbook.start(tx2).await.unwrap();
    });
    tokio::spawn(async move {
        let mut exchange_to_orderbook = HashMap::<ExchangeName, OrderBookOnlyLevels>::new();
        loop {
            select! {
                // Receive a one shot sender that sends a summary receiver back to the server.
                val = rx3.recv() => {
                    if let Some(oneshot_sender) = val {
                        oneshot_sender.send(tx4.subscribe()).unwrap();
                        tracing::info!("rx count: {}", tx4.receiver_count());
                    }
                },
                // Receive book levels from the order books.
                val = rx.recv() => {
                    if let Some(orderbook) = val {
                        match orderbook {
                            OrderBookOnlyLevels { exchange: ExchangeName::BITSTAMP, .. }  => {
                                exchange_to_orderbook.insert(ExchangeName::BITSTAMP, orderbook);
                                println!("bitstamp!");
                            }
                            OrderBookOnlyLevels { exchange: ExchangeName::BINANCE, .. } => {
                                exchange_to_orderbook.insert(ExchangeName::BINANCE, orderbook);
                                // println!("exchange_to_orderbook: {:?}", exchange_to_orderbook);
                                println!("binance!");
                            }
                        }
                        if exchange_to_orderbook.len() < 2 {
                            println!("continue");
                            continue;
                        }
                        
                        // println!("{:?}", exchange_to_orderbook.values()); // 
                        // println!("{:?}", exchange_to_orderbook.values().len()); // 2
                        // Book levels are stored in the hashmap above and a new summary created from both exchanges
                        // every time an update is received from either.
                        let current_levels = exchange_to_orderbook.values().map(|v| v.clone()).collect::<Vec<OrderBookOnlyLevels>>();
                        // println!("{:?}", current_levels);
                        // println!("current_levels len {:?}", current_levels.len());
                        let summary = make_summary(current_levels, symbol);

                        tx4.send_replace(Ok(summary)).unwrap();
                    }
                },
            }
        }
    });
    tx3
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = tracing_subscriber::fmt()
        .with_line_number(true)
        .with_max_level(tracing::Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let config = Config::builder()
        // read the setting.toml
        .add_source(config::File::with_name("orderbook-merger/src/setting"))
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