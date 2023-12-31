use crate::{*, orderbook_summary::Level};
use anyhow::Result;
use std::collections::BTreeMap;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

// The `OrderBookOnlyLevels` struct represents an order book with bids and asks for a specific symbol
// on an exchange.
#[derive(Debug, Default, Clone)]
pub struct OrderBookOnlyLevels {
    pub exchange: ExchangeName,
    pub symbol: Symbol,
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
    pub last_update_id: u64,
}

// The `Update` trait defines methods that should be implemented by types that represent
// updates to an orderbook.
pub trait Update {
    fn validate(&self, last_id: u64) -> Result<()>;
    fn last_update_id(&self) -> u64;
    fn bids_mut(&mut self) -> &mut BTreeMap<DisplayAmount, DisplayAmount>;
    fn asks_mut(&mut self) -> &mut BTreeMap<DisplayAmount, DisplayAmount>;
}

// The `OrderBook` struct represents an orderbook for a specific exchange and symbol.
pub struct OrderBook {
    pub exchange: ExchangeName,
    pub symbol: Symbol,
    pub price_scale: u32,
    pub quantity_scale: u32,
    pub bids: BTreeMap<StorageAmount, StorageAmount>,
    pub asks: BTreeMap<StorageAmount, StorageAmount>,
    pub last_update_id: u64,
}

impl OrderBook {
    // The `new_orderbook` function creates a new orderbook with empty bids and asks maps, and
    // initializes other fields.
    pub fn new_orderbook(
        exchange: ExchangeName,
        symbol: Symbol,
        price_scale: u32,
        quantity_scale: u32,
    ) -> Self {
        let bids: BTreeMap<StorageAmount, StorageAmount> = BTreeMap::new();
        let asks: BTreeMap<StorageAmount, StorageAmount> = BTreeMap::new();

        Self {
            exchange,
            symbol,
            price_scale,
            quantity_scale,
            bids,
            asks,
            last_update_id: u64::MIN
        }
    }

    // The `storage_to_display` function takes in storage_level and converts them to
    // a displayable format by rounding and scaling the price and quantity.
    fn storage_to_display(&self, storage_level: [StorageAmount; 2]) -> Result<Level> {
        let price = (storage_level[0].to_display(self.price_scale)?.to_f64().unwrap()
            * 10u32.pow(self.price_scale) as f64)
            .round()
            / 10u32.pow(self.price_scale) as f64;
        let quantity = (storage_level[1].to_display(self.quantity_scale)?.to_f64().unwrap()
            * 10u32.pow(self.quantity_scale) as f64)
            .round()
            / 10u32.pow(self.quantity_scale) as f64;
        let level = Level {
            exchange: self.exchange.to_string(),
            price,
            quantity,
        };

        Ok(level)
    }

    fn bids(&self) -> &BTreeMap<StorageAmount, StorageAmount> {
        &self.bids
    }

    fn bids_mut(&mut self) -> &mut BTreeMap<StorageAmount, StorageAmount> {
        &mut self.bids
    }

    fn asks(&self) -> &BTreeMap<StorageAmount, StorageAmount> {
        &self.asks
    }

    fn asks_mut(&mut self) -> &mut BTreeMap<StorageAmount, StorageAmount> {
        &mut self.asks
    }
    
    pub fn add_bid(&mut self, level: [Decimal; 2]) -> Result<()> {
        let price = level[0].to_storage(self.price_scale)?;
        let quantity = level[1].to_storage(self.quantity_scale)?;
        
        let bids = self.bids_mut();
        if quantity > 0 {
            bids.insert(price, quantity);
        } else {
            bids.remove(&price);
        }

        Ok(())
    }

    pub fn add_ask(&mut self, level: [Decimal; 2]) -> Result<()> {
        let price = level[0].to_storage(self.price_scale)?;
        let quantity = level[1].to_storage(self.quantity_scale)?;

        let asks = self.asks_mut();
        if quantity > 0 {
            asks.insert(price, quantity);
        } else {
            asks.remove(&price);
        }

        Ok(())
    }
    pub fn get_bids_levels(&self) -> Result<Vec<Level>> {
        let bids = self.bids();
        
        let summary_bids = if bids.is_empty() {
            Vec::new()
        } else {
            let mut summary_bids = Vec::<Level>::with_capacity(10);
            for (&price, &quantity) in bids.iter().rev().take(10) {
                let level = self.storage_to_display([price, quantity])?;
                summary_bids.push(level);
            }
            summary_bids
        };
        Ok(summary_bids)
    }
    pub fn get_asks_levels(&self) -> Result<Vec<Level>> {
        let asks = self.asks();
        let summary_asks = if asks.is_empty() {
            Vec::new()
        } else {
            let mut summary_asks = Vec::<Level>::with_capacity(10);
            for (&price, &quantity) in asks.iter().take(10) {
                let level = self.storage_to_display([price, quantity])?;
                summary_asks.push(level);
            }
            summary_asks
        };
        Ok(summary_asks)
    }
    pub fn get_book_levels(&self) -> Option<OrderBookOnlyLevels> {
        // levels come out here with the best bid and ask at the end of the vector
        let bids = self.get_bids_levels().ok()?;
        let asks = self.get_asks_levels().ok()?;
        if bids.is_empty() && asks.is_empty() {
            None
        } else {
            Some(OrderBookOnlyLevels {
                exchange: self.exchange,
                symbol: self.symbol,
                last_update_id: self.last_update_id,
                bids,
                asks,
            })
        }
    }
    pub fn update<U: Update + std::fmt::Debug>(&mut self, update: &mut U) -> Result<()> {
        // tracing::debug!("update {:#?}", update);

        update.validate(self.last_update_id)?;

        for (price, quantity) in update.bids_mut().iter() {
            // tracing::debug!("adding bid: {:?}", [*price, *quantity]);
            self.add_bid([*price, *quantity])?;
        }
        for (price, quantity) in update.asks_mut().iter() {
            // tracing::debug!("adding ask: {:?}", [*price, *quantity]);
            self.add_ask([*price, *quantity])?;
        }
        tracing::debug!("Update done!");
        self.last_update_id = update.last_update_id();
        Ok(())
    }
}