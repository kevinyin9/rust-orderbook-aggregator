use anyhow::{Context, Result};
use crate::{orderbook::orderbook::{OrderBook, OrderBookBasicInfo}};
use super::exchange::{Exchange, ExchangeInfo};
use serde::{Deserialize, Deserializer, Serialize};
use std::sync::Arc;
use serde_json::Value;
use tokio::sync::Mutex;
use async_trait::async_trait;
use url::Url;
use crate::{Symbol, ExchangeName};

fn from_str<'de, D>(deserializer: D) -> Result<Vec<[f64; 2]>, D::Error>
where
    D: Deserializer<'de>,
{
    let v: Vec<[&str; 2]> = Deserialize::deserialize(deserializer)?;
    Ok(v.into_iter()
        .map(|s| (s[0].parse::<f64>(), s[1].parse::<f64>()))
        .filter_map(|p| Some([p.0.ok()?, p.1.ok()?]))
        .collect::<Vec<[f64; 2]>>())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub last_update_id: u64,
    #[serde(deserialize_with = "from_str")]
    pub bids: Vec<[f64; 2]>,
    #[serde(deserialize_with = "from_str")]
    pub asks: Vec<[f64; 2]>,
}

pub struct Binance {
    pub orderbook: Arc<Mutex<OrderBook>>,
}

#[async_trait]
impl Exchange for Binance {

    const BASE_URL_HTTPS: &'static str = "https://www.binance.us/api/v3/";
    const BASE_URL_WSS: &'static str = "wss://stream.binance.us:9443/ws/";

    async fn new_exchange(symbol: Symbol) -> Result<Self>
    {
        let exchange_name = ExchangeName::BINANCE;
        let orderbook = Self::new_orderbook(exchange_name, symbol).await?;
        let binance = Self {
            orderbook: Arc::new(Mutex::new(orderbook)),
        };
        Ok(binance)
    }

    async fn get_exchange_info(url: Url, symbol: Symbol) -> Result<ExchangeInfo> {
        let mut endpoint = url.join("exchangeInfo").unwrap();
        endpoint
            .query_pairs_mut()
            .append_pair("symbol", &symbol.to_string())
            .finish();

        let exchange_info = reqwest::get(endpoint)
            .await
            .context("Failed to get exchange info")?
            .json::<ExchangeInfo>()
            .await
            .context("Failed to deserialize exchange info to json")?;
        Ok(exchange_info)
    }

    async fn get_orderbook_info(symbol: &Symbol) -> Result<OrderBookBasicInfo> {
        let (best_price, _) = (100.0, 0);
        // Self::fetch_prices(symbol).await?;

        // println!("base_url_https: {}", Self::base_url_https());
        let (price_precision, quantity_precision) = (8.0, 9.0);
            // ExchangeInfoBinance::fetch_scales(Self::base_url_https(), symbol).await?;
        let (price_min, price_max) = (3.0, 5.0);
            // OrderBookBasicInfo::get_min_max(best_price, price_range, price_precision)?;

        let args = OrderBookBasicInfo {
            price_min,
            price_max,
            price_precision,
            quantity_precision,
        };

        Ok(args)
    }
}