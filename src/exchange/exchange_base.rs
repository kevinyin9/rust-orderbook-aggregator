use url::Url;

pub trait Exchange {
    const BASE_URL_HTTPS: &'static str;
    const BASE_URL_WSS: &'static str;

    async fn new_exchange(symbol: String) {
    }

    async fn new_orderbook(exchange: String, symbol: String) {
    }
}
