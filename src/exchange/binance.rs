use crate::{orderbook::order_book::OrderBook};

pub struct Binance {
    pub orderbook: Arc<Mutex<OrderBook>>,
}

impl Exchange for Binance {
    async fn new(symbol: &str) -> Result<Self>
    {
        let exchange = Exchange::BINANCE;
        let orderbook = Self::new_orderbook(exchange, symbol).await?;
        let exchange_orderbook = Self {
            orderbook: Arc::new(Mutex::new(orderbook)),
        };
        Ok(exchange_orderbook)
    }

    async fn get_orderbook_info(symbol: &str) -> Result<OrderBookArgs> {
        let (best_price, _) = Self::fetch_prices(symbol).await?;

        println!("base_url_https: {}", Self::base_url_https());
        let (scale_price, scale_quantity) =
            ExchangeInfoBinance::fetch_scales(Self::base_url_https(), symbol).await?;
        let (storage_price_min, storage_price_max) =
            OrderBookArgs::get_min_max(best_price, price_range, scale_price)?;

        let args = OrderBookArgs {
            storage_price_min,
            storage_price_max,
            scale_price,
            scale_quantity,
        };

        tracing::debug!("orderbook args: {:#?}", args);

        Ok(args)
    }
}