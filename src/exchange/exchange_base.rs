use url::Url;
use crate::order_book::orderbook::{OrderBook};

pub trait Exchange {
    const BASE_URL_HTTPS: &'static str;
    const BASE_URL_WSS: &'static str;

    async fn new_exchange(symbol: &str) -> Result<>;

    async fn new_orderbook(exchange: &str, symbol: &str) {
        let OrderBookBasicInfo {
            storage_price_min,
            storage_price_max,
            scale_price,
            scale_quantity,
        } = Self::get_orderbook_info(&symbol, price_range).await?;

        let orderbook = OrderBook::new(
            exchange,
            symbol,
            storage_price_min,
            storage_price_max,
            scale_price,
            scale_quantity,
        );

        tracing::debug!(
            "returning orderbook for {} {} min: {} max: {} scale_p: {}, scale_q: {}",
            exchange,
            symbol,
            storage_price_min,
            storage_price_max,
            scale_price,
            scale_quantity
        );
        Ok(orderbook)
    }

    async fn get_orderbook_info(symbol: &Symbol, price_range: u8) -> Result<OrderBookArgs>;
}
