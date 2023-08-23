use crate::{orderbook::orderbook::{OrderBook, OrderBookBasicInfo}};
use super::exchange_base::{Exchange};
use std::sync::Arc;
use tokio::sync::Mutex;
use async_trait::async_trait;

pub struct Binance {
    pub orderbook: Arc<Mutex<OrderBook>>,
}

#[async_trait]
impl Exchange for Binance {
    async fn new_exchange(symbol: String) -> Result<Self>
    {
        let exchange = "Binance";
        let orderbook = Self::new_orderbook(exchange, symbol).await?;
        let exchange_orderbook = Self {
            orderbook: Arc::new(Mutex::new(orderbook)),
        };
        Ok(exchange_orderbook)
    }

    async fn get_orderbook_info(symbol: String) -> Result<OrderBookBasicInfo> {
        let (best_price, _) = Self::fetch_prices(symbol).await?;

        println!("base_url_https: {}", Self::base_url_https());
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