use anyhow::{ensure, Context, Result};
use crate::{Symbol, ExchangeName, orderbook::orderbook::{OrderBook, Update}};
use super::exchange::Exchange;
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    collections::BTreeMap,
    sync::Arc,
};
use serde_json::Value;
use rust_decimal::Decimal;
use std::str::FromStr;
use tokio::sync::Mutex;
use async_trait::async_trait;
use url::Url;
use tokio::net::TcpStream;
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
#[serde(rename_all = "camelCase")]
pub(super) struct SymbolData {
    pub symbol: String,
    pub base_asset_precision: u32,
    pub quote_asset_precision: u32,
    pub filters: Vec<Value>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
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
#[serde(rename_all = "camelCase")]
pub struct BookUpdate {
    #[serde(alias = "U")]
    pub first_update_id: u64,
    #[serde(alias = "u")]
    pub last_update_id: u64,
    #[serde(alias = "b", deserialize_with = "from_str")]
    pub bids: BTreeMap<Decimal, Decimal>,
    #[serde(alias = "a", deserialize_with = "from_str")]
    pub asks: BTreeMap<Decimal, Decimal>,
}

impl Update for BookUpdate {
    fn validate(&self, last_id: u64) -> Result<()> {
        let first_update_id = self.first_update_id;
        if last_id == 0 {
            return Ok(());
        }
        // println!("{} {}", first_update_id, last_id + 1);
        ensure!(
            first_update_id == last_id + 1,
            "first_update_id: {first_update_id} != last_id: {last_id} + 1"
        );
        // println!("wtf2");
        ensure!(
            last_id < self.first_update_id,
            "last_id: {last_id} >= first_update_id: {first_update_id}"
        );
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

impl TryFrom<Message> for BookUpdate {
    type Error = anyhow::Error;
    fn try_from(item: Message) -> Result<Self> {
        serde_json::from_slice::<Self>(&item.into_data()).context("Failed to deserialize update")
    }
}

impl From<Snapshot> for BookUpdate {
    fn from(snapshot: Snapshot) -> Self {
        Self {
            first_update_id: 1,
            last_update_id: snapshot.last_update_id,
            bids: snapshot.bids,
            asks: snapshot.asks,
        }
    }
}

pub struct Binance {
    pub orderbook: Arc<Mutex<OrderBook>>,
}

#[async_trait]
impl Exchange<Snapshot, BookUpdate> for Binance {

    const BASE_URL_HTTPS: &'static str = "https://www.binance.us/api/v3/";
    const BASE_URL_WSS: &'static str = "wss://stream.binance.us:9443/ws/";

    fn orderbook(&self) -> Arc<Mutex<OrderBook>> {
        self.orderbook.clone()
    }

    async fn new_exchange(symbol: Symbol) -> Result<Self>
    {
        let orderbook = Self::new_orderbook(ExchangeName::BINANCE, symbol).await?;
        Ok(
            Self {
                orderbook: Arc::new(Mutex::new(orderbook))
            }
        )
    }

    async fn get_scales(symbol: &Symbol) -> Result<(u32, u32)> {
        let url = Self::base_url_https();
        let mut endpoint = url.join("exchangeInfo").unwrap();
        endpoint
            .query_pairs_mut()
            .append_pair("symbol", &symbol.to_string())
            .finish();

        let exchange_info = reqwest::get(endpoint)
            .await
            .context("Failed to get exchange info")?
            .json::<SymbolData>()
            .await
            .context("Failed to deserialize exchange info to json")?;
        
        let tick_sizes = exchange_info
            .filters
            .iter()
            .filter_map(|filter| {
                let filter_obj = filter.as_object()?;
                if let Some(filter_type) = filter["filterType"].as_str() {
                    if filter_type == "PRICE_FILTER" {
                        filter_obj["tickSize"].as_str()
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<&str>>();

        let tick_size_str = tick_sizes.first().context("Failed to get tick size")?;

        let price_scale = Decimal::from_str(tick_size_str)
            .context("Failed to parse tick size")?
            .normalize()
            .scale();

        let quantity_scale = exchange_info.base_asset_precision.min(8);

        Ok((price_scale, quantity_scale))
    }

    async fn get_snapshot(&self) -> Result<Snapshot> {
        let symbol = self.orderbook().lock().await.symbol;
        let mut url = Self::base_url_https().join("depth").unwrap();
        url.query_pairs_mut()
            .append_pair("symbol", &symbol.to_string())
            .append_pair("limit", "1000")
            .finish();
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
        let endpoint = format!("{}@depth@100ms", symbol);
        let url = Self::base_url_wss().join(&endpoint).unwrap();
        let (stream, _) = connect_async(url)
            .await
            .context("Failed to connect to wss endpoint")?;
        Ok(stream)
    }
}