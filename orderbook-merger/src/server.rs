use anyhow::Result;
use config::Config;
use std::collections::HashMap;
use tokio::{sync::mpsc, sync::watch};
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
    summary: watch::Receiver<Result<Summary, Status>>
}

#[tonic::async_trait]
impl OrderbookAggregator for OrderbookSummary {
    type BookSummaryStream = WatchStream<Result<Summary, Status>>;
    async fn book_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        Ok(Response::new(WatchStream::new(self.summary.clone())))
    }
}

async fn aggregate_and_broadcast_data(mut rx: mpsc::Receiver<OrderBookOnlyLevels>, tx_summary: watch::Sender<Result<Summary, Status>>) {
    let mut exchange_to_orderbook = HashMap::<ExchangeName, OrderBookOnlyLevels>::new();
    while let Some(orderbook) = rx.recv().await {
        match orderbook {
            OrderBookOnlyLevels { exchange: ExchangeName::BITSTAMP, .. } => {
                exchange_to_orderbook.insert(ExchangeName::BITSTAMP, orderbook);
            }
            OrderBookOnlyLevels { exchange: ExchangeName::BINANCE, .. } => {
                exchange_to_orderbook.insert(ExchangeName::BINANCE, orderbook);
            }
        }

        // Book levels are stored in the hashmap above and a new summary created from both exchanges
        // every time an update is received from either.
        if exchange_to_orderbook.len() == 2 {
            // let current_levels = exchange_to_orderbook.values().map(|v| v.clone()).collect::<Vec<OrderBookOnlyLevels>>();
            let current_levels: Vec<_> = exchange_to_orderbook.values().cloned().collect();
            let summary = make_summary(current_levels);
            tx_summary.send_replace(Ok(summary)).unwrap();
        }
    }
}

async fn start(symbol: Symbol) -> watch::Receiver<Result<Summary, Status>>{
    let (tx_orderbook, rx_orderbook) = mpsc::channel::<OrderBookOnlyLevels>(20);
    let tx2_orderbook = tx_orderbook.clone();

    let (tx_summary, rx_summary) = watch::channel(Ok(Summary::default()));

    let binance = Binance::new_exchange(symbol).await.unwrap();
    let bitstamp = Bitstamp::new_exchange(symbol).await.unwrap();

    tokio::spawn(async move {
        binance.start(tx_orderbook).await.unwrap()
    });
    tokio::spawn(async move {
        bitstamp.start(tx2_orderbook).await.unwrap()
    });
    tokio::spawn(
        aggregate_and_broadcast_data(rx_orderbook, tx_summary)
    );

    rx_summary
}

#[tokio::main]
async fn main() -> Result<()> {
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
        summary: start(Symbol::ETHUSDT).await,
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