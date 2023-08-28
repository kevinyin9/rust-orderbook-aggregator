use anyhow::{Context, Result};
use url::Url;
use crate::orderbook::orderbook::{OrderBook, OrderBookOnlyLevels, Update};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
use async_trait::async_trait;
use tokio_stream::StreamExt;
use tokio::{
    net::TcpStream,
    sync::{mpsc, Mutex},
    task::JoinHandle,
};
use rust_decimal::Decimal;
use std::sync::Arc;
use crate::{Symbol, ExchangeName};

#[async_trait]
pub trait Exchange<
    S: Update + Send,
    U: std::fmt::Debug
    + Update
    + From<S>
    + TryFrom<Message, Error = anyhow::Error>
    + Send
    + Sync
    + 'static,
    Error = anyhow::Error,>
{
    const BASE_URL_HTTPS: &'static str;
    const BASE_URL_WSS: &'static str;

    fn base_url_https() -> Url {
        Url::parse(&Self::BASE_URL_HTTPS).unwrap()
    }
    fn base_url_wss() -> Url {
        Url::parse(&Self::BASE_URL_WSS).unwrap()
    }
    fn orderbook(&self) -> Arc<Mutex<OrderBook>>;

    async fn new_exchange(symbol: Symbol) -> Result<Self>
    where
        Self: Sized;

    async fn new_orderbook(exchange: ExchangeName, symbol: Symbol)  -> Result<OrderBook>
    where
        Self: Sized,
    {
        let orderbook = OrderBook::new_orderbook(
            exchange,
            symbol
        );

        println!(
            "returning orderbook for {} {}",
            exchange,
            symbol
        );
        Ok(orderbook)
    }

    // async fn get_price(symbol: &Symbol) -> Result<(Decimal, Decimal)>;
    async fn get_tick_price(symbol: &Symbol) -> Result<(Decimal, Decimal)>;
    async fn get_snapshot(&self) -> Result<S>;
    async fn get_websocket_stream(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>>;

    async fn start(&self, tx_summary: mpsc::Sender<OrderBookOnlyLevels>) -> Result<()> {
        let (tx_update, mut rx_update) = mpsc::channel::<U>(100);

        let mut websocket_stream = self.get_websocket_stream().await?;
        let snapshot = self.get_snapshot().await?;
        let snapshot_update = U::from(snapshot);

        let fetcher: JoinHandle<std::result::Result<(), anyhow::Error>> =
            tokio::spawn(async move {
                tx_update
                    .send(snapshot_update)
                    .await
                    .context("failed to send snapshot")?;
                
                while let Some(response) = websocket_stream.next().await {
                    match response {
                        Ok(message) => match U::try_from(message) {
                            Ok(mut update) => {
                                tx_update
                                    .send(update)
                                    .await
                                    .context("failed to send update")?;
                            }
                            Err(_) => {
                                // println!("failed1!");
                                continue;    
                            }
                        },
                        Err(e) => {
                            // println!("failed2!");
                            continue;
                        }
                    }
                }
                Ok(())
            });
        let orderbook = self.orderbook();
        while let Some(mut update) = rx_update.recv().await {
            let mut ob = orderbook.lock().await;
            let exchange = ob.exchange;
            let symbol = ob.symbol;
            if let Err(err) = ob.update(&mut update) {
                // println!("{} failed3!", exchange);
                continue;
            } else {
                if let Some(book_levels) = ob.get_book_levels() {
                    println!("{} send", exchange);
                    tx_summary.send(book_levels.clone()).await.unwrap();
                }
            }
        }
        let _ = fetcher.await?;
        Ok(())
    }
}
