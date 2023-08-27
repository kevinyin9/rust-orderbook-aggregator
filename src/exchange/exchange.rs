use anyhow::{Context, Result};
use url::Url;
use serde::{Deserialize, Deserializer, Serialize};
use crate::orderbook::orderbook::{OrderBook, OrderBookBasicInfo, OrderBookOnlyLevels, Update};
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
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ExchangeInfo {
    pub symbol: String,
    pub base_asset_precision: u32,
    pub quote_asset_precision: u32,
    pub filters: Vec<Value>,
}

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
        let OrderBookBasicInfo {
            price_precision,
            quantity_precision,
            price_min,
            price_max,
        } = Self::get_orderbook_info(&symbol).await?;

        let orderbook = OrderBook::new_orderbook(
            exchange,
            symbol,
            price_precision,
            quantity_precision,
            price_min,
            price_max
        );

        println!(
            "returning orderbook for {} {} min: {} max: {} scale_p: {}, scale_q: {}",
            exchange,
            symbol,
            price_precision,
            quantity_precision,
            price_min,
            price_max
        );
        Ok(orderbook)
    }

    async fn get_orderbook_info(symbol: &Symbol) -> Result<OrderBookBasicInfo>;
    async fn get_tick_price(symbol: &Symbol) -> Result<(Decimal, Decimal)>;
    async fn get_exchange_info(url: Url, symbol: Symbol) -> Result<ExchangeInfo>;
    async fn get_snapshot(&self) -> Result<S>;
    async fn get_websocket_stream(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>>;

    async fn start(&self, tx_summary: mpsc::Sender<OrderBookOnlyLevels>) -> Result<()> {
        let (tx_update, mut rx_update) = mpsc::channel::<U>(100);

        let snapshot = self.get_snapshot().await?;
        let mut websocket_stream = self.get_websocket_stream().await?;
        let snapshot_update = U::from(snapshot);
        println!("hello!");
        let fetcher: JoinHandle<std::result::Result<(), anyhow::Error>> =
            tokio::spawn(async move {
                println!("hello222!");
                tx_update
                    .send(snapshot_update)
                    .await
                    .context("failed to send snapshot")?;
                println!("hello3333!"); 
                while let Some(response) = websocket_stream.next().await {
                    match response {
                        Ok(message) => match U::try_from(message) {
                            Ok(mut update) => {
                                println!("hello444!");
                                tx_update
                                    .send(update)
                                    .await
                                    .context("failed to send update")?;
                                println!("success send!");
                            }
                            Err(_) => {
                                println!("hello555!");
                                continue;    
                            }
                        },
                        Err(e) => {
                            println!("hello666!");
                            continue;
                        }
                    }
                }
                Ok(())
            });
        println!("hello777!");
        let orderbook = self.orderbook();
        println!("hello888!");
        while let Some(mut update) = rx_update.recv().await {
            println!("hello999!");
            let mut ob = orderbook.lock().await;
            let exchange = ob.exchange;
            let symbol = ob.symbol;
            println!("hello1111!");
            if let Err(err) = ob.update(&mut update) {
                continue;
            } else {
                if let Some(book_levels) = ob.get_book_levels() {
                    println!("hello324234234!");
                    tx_summary.send(book_levels.clone()).await.unwrap();
                    println!("success send666!");
                }
            }
            println!("hello5553335!");
        }
        println!("hello4252342!");
        let _ = fetcher.await?;
        println!("hello67657!");
        Ok(())
    }
}
