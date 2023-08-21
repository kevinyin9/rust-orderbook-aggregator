pub struct OrderBookBasicInfo {
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub bids: Vec<u64>,
    pub asks: Vec<u64>,
    pub price_precision: f64,
    pub quantity_precision: f64,
    pub last_update_id: u64,
    pub price_min: u64,
    pub price_max: u64,
}

impl OrderBookBasicInfo {
    pub fn new(
        exchange: u64,
        symbol: Symbol,
        price_precision: u64,
        quantity_precision: u64,
        price_min: u64,
        price_max: u64,
    ) -> Self {
        let capacity: u32 = 10;
        let mut bids: Vec<u64> = vec![0; capacity];
        let mut asks: Vec<u64> = vec![0; capacity];

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