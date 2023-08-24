use anyhow::{Context, Result};
use url::Url;
use serde::{Deserialize, Deserializer, Serialize};
use crate::orderbook::orderbook::{OrderBook, OrderBookBasicInfo};
use async_trait::async_trait;
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
pub trait Exchange {
    const BASE_URL_HTTPS: &'static str;
    const BASE_URL_WSS: &'static str;

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
    async fn get_exchange_info(url: Url, symbol: Symbol) -> Result<ExchangeInfo>;
}
