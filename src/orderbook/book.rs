
pub struct OrderBookBasicInfo {
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub bids: Vec<f64>,
    pub asks: Vec<f64>,
    pub price_precision: f64,
    pub quantity_precision: f64,
    pub last_update_id: u64,
    pub price_min: f64,
    pub price_max: f64,
}

impl OrderBookBasicInfo {
    pub fn new(
        exchange: f64,
        symbol: Symbol,
        price_precision: f64,
        quantity_precision: f64,
        price_min: f64,
        price_max: f64,
    ) -> Self {
        let capacity: u32 = 10;
        let mut bids: Vec<f64> = vec![0; capacity];
        let mut asks: Vec<f64> = vec![0; capacity];

        Self {
            exchange,
            symbol,
            bids,
            asks,
            price_precision,
            quantity_precision,
            last_update_id: u64::MIN,
            price_min,
            price_max
        }
    }
}