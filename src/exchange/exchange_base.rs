use anyhow::{Context, Result};
use url::Url;
use crate::orderbook::orderbook::{OrderBook, OrderBookBasicInfo};
use async_trait::async_trait;

#[async_trait]
pub trait Exchange<Error = anyhow::Error> {
    const BASE_URL_HTTPS: &'static str;
    const BASE_URL_WSS: &'static str;

    async fn new_exchange(symbol: String) -> Result<Self>
    where
        Self: Sized;

    async fn new_orderbook(exchange: String, symbol: String)  -> Result<OrderBook>
    where
        Self: Sized,
    {
        let OrderBookBasicInfo {
            price_precision,
            quantity_precision,
            price_min,
            price_max,
        } = Self::get_orderbook_info(&symbol).await?;

        let orderbook = OrderBook::new(
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

    async fn get_orderbook_info(symbol: String, price_range: u8) -> Result<OrderBookBasicInfo>;
}
