use anyhow::{Context, Result};
use crate::{DisplayAmount, Symbol, ExchangeName, orderbook::orderbook::{OrderBook, Update}};
use super::exchange::Exchange;
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    collections::BTreeMap,
    sync::Arc,
};
use futures::SinkExt;
use rust_decimal::Decimal;
use tokio::sync::Mutex;
use async_trait::async_trait;
use url::Url;
use tokio::net::TcpStream;
use serde_aux::field_attributes::deserialize_number_from_string;
use tokio_tungstenite::{tungstenite::Message, connect_async, MaybeTlsStream, WebSocketStream};

fn from_str<'de, D>(deserializer: D) -> Result<BTreeMap<Decimal, Decimal>, D::Error>
where
    D: Deserializer<'de>,
{
    let v: Vec<[&str; 2]> = Deserialize::deserialize(deserializer)?;
    let mut map = BTreeMap::new();
    for s in v {
        match (s[0].parse::<Decimal>(), s[1].parse::<Decimal>()) {
            (Ok(key), Ok(value)) => {
                map.insert(key, value);
            }
            _ => return Err(serde::de::Error::custom("Failed to parse Decimal")),
        }
    }
    Ok(map)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct SymbolData {
    pub url_symbol: String,
    pub base_decimals: u32,
    pub counter_decimals: u32,
    pub instant_order_counter_decimals: u32,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Snapshot {
    #[serde(
        alias = "microtimestamp",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub last_update_id: u64,
    #[serde(deserialize_with = "from_str")]
    pub bids: BTreeMap<Decimal, Decimal>,
    #[serde(deserialize_with = "from_str")]
    pub asks: BTreeMap<Decimal, Decimal>,
}

impl Update for Snapshot {
    fn validate(&self, _: u64) -> Result<()> {
        Ok(())
    }
    fn last_update_id(&self) -> u64 {
        self.last_update_id
    }
    fn bids_mut(&mut self) -> &mut BTreeMap<Decimal, Decimal> {
        &mut self.bids
    }

    fn asks_mut(&mut self) -> &mut BTreeMap<Decimal, Decimal> {
        &mut self.asks
    }
}

impl Snapshot {
    pub(crate) async fn fetch(url: Url) -> Result<Self> {
        let snapshot = reqwest::get(url)
            .await
            .context("Failed to get snapshot")?
            .json::<Self>()
            .await
            .context("Failed to deserialize snapshot")?;
        Ok(snapshot)
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BookUpdateData {
    #[serde(
        alias = "microtimestamp",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub last_update_id: u64,
    #[serde(alias = "b", deserialize_with = "from_str")]
    pub bids: BTreeMap<DisplayAmount, DisplayAmount>,
    #[serde(alias = "a", deserialize_with = "from_str")]
    pub asks: BTreeMap<DisplayAmount, DisplayAmount>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BookUpdate {
    data: BookUpdateData,
}

impl Update for BookUpdate {
    fn validate(&self, _: u64) -> Result<()> {
        Ok(())
    }
    fn last_update_id(&self) -> u64 {
        self.data.last_update_id
    }
    fn bids_mut(&mut self) -> &mut BTreeMap<DisplayAmount, DisplayAmount> {
        &mut self.data.bids
    }

    fn asks_mut(&mut self) -> &mut BTreeMap<DisplayAmount, DisplayAmount> {
        &mut self.data.asks
    }
}

impl TryFrom<Message> for BookUpdate {
    type Error = anyhow::Error;
    fn try_from(item: Message) -> Result<Self> {
        match serde_json::from_slice::<Self>(&item.into_data()) {
            Ok(update) => Ok(update),
            Err(e) => {
                tracing::error!("Failed to deserialize update: {}", e);
                Err(anyhow::Error::new(e).context("Failed to deserialize update"))
            }
        }
    }
}

impl From<Snapshot> for BookUpdate {
    fn from(snapshot: Snapshot) -> Self {
        println!("bitstamp ok123");
        Self {
            data: BookUpdateData {
                last_update_id: snapshot.last_update_id,
                bids: snapshot.bids,
                asks: snapshot.asks,
            },
        }
    }
}

pub struct Bitstamp {
    pub orderbook: Arc<Mutex<OrderBook>>,
}

#[async_trait]
impl Exchange<Snapshot, BookUpdate> for Bitstamp {

    const BASE_URL_HTTPS: &'static str = "https://www.bitstamp.net/api/v2/";
    const BASE_URL_WSS: &'static str = "wss://ws.bitstamp.net/";

    fn orderbook(&self) -> Arc<Mutex<OrderBook>> {
        self.orderbook.clone()
    }
    async fn new_exchange(symbol: Symbol) -> Result<Self>
    {
        let orderbook = Self::new_orderbook(ExchangeName::BITSTAMP, symbol).await?;
        Ok(
            Self {
                orderbook: Arc::new(Mutex::new(orderbook))
            }
        )
    }

    async fn get_scales(symbol: &Symbol) -> Result<(u32, u32)> {
        let url = Self::base_url_https();
        let endpoint = url.join("trading-pairs-info").unwrap();
        
        let symbols = reqwest::get(endpoint)
            .await
            .context("Failed to get exchange info")?
            .json::<Vec<SymbolData>>()
            .await
            .context("Failed to deserialize exchange info to json")?;

        let symbol = symbols
            .into_iter()
            .filter(|s| s.url_symbol == symbol.to_string().to_lowercase())
            .next()
            .context("Failed to get symbol")?;

        let price_scale = symbol.counter_decimals;
        let quantity_scale = symbol.base_decimals;

        Ok((price_scale, quantity_scale))
    }

    async fn get_snapshot(&self) -> Result<Snapshot> {
        let symbol = self.orderbook().lock().await.symbol.to_string().to_lowercase();
        let url = Self::base_url_https().join(format!("order_book/{}", symbol).as_str())?;
        Snapshot::fetch(url).await
    }

    async fn get_websocket_stream(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        let symbol = self
            .orderbook()
            .lock()
            .await
            .symbol
            .to_string()
            .to_lowercase();
        let subscribe_msg = serde_json::json!({
            "event": "bts:subscribe",
            "data": {
                "channel": format!("diff_order_book_{}", symbol)
            }
        });

        // let (mut stream, _) = connect_async(&Self::base_url_wss())
        //     .await
        //     .context("Failed to connect to bit stamp wss endpoint")?;

        // stream
        //     .start_send_unpin(Message::Text(subscribe_msg.to_string()))
        //     .context("Failed to send subscribe message to bitstamp")?;
        let mut stream = match connect_async(&Self::base_url_wss()).await.context("Failed to connect to bitstamp wss endpoint") {
            Ok((stream, _)) => stream,
            Err(e) => {
                eprintln!("Error: {:?}", e); // Print the error with its context
                return Err(e);
            }
        };
        
        match stream.start_send_unpin(Message::Text(subscribe_msg.to_string())) {
            Ok(_) => {
                // Successfully sent the message.
            },
            Err(e) => {
                eprintln!("Error: {:?}", e);  // Print the error with its context
                return Err(e.into());
            }
        }

        Ok(stream)
    }
}