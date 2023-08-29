use crate::{Symbol, ExchangeName, orderbook_summary::Level};
use anyhow::Result;
use std::collections::BTreeMap;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

#[derive(Debug, Default, Clone)]
pub struct OrderBookBasicInfo {
    pub price_precision: Decimal,
    pub quantity_precision: Decimal,
}

#[derive(Debug, Clone)]
pub struct OrderBookOnlyLevels {
    pub exchange: ExchangeName,
    pub symbol: Symbol,
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
    pub last_update_id: u64,
}

/// Updates from all exchanges should implement this trait
pub trait Update {
    fn validate(&self, last_id: u64) -> Result<()>;
    fn last_update_id(&self) -> u64;
    fn bids_mut(&mut self) -> &mut BTreeMap<Decimal, Decimal>;
    fn asks_mut(&mut self) -> &mut BTreeMap<Decimal, Decimal>;
}

pub struct OrderBook {
    pub exchange: ExchangeName,
    pub symbol: Symbol,
    pub bids: BTreeMap<Decimal, Decimal>,
    pub asks: BTreeMap<Decimal, Decimal>,
    pub last_update_id: u64,
}

impl OrderBook {
    pub fn new_orderbook(
        exchange: ExchangeName,
        symbol: Symbol,
    ) -> Self {
        let capacity: usize = 10;
        let mut bids: BTreeMap<Decimal, Decimal> = BTreeMap::new();
        let mut asks: BTreeMap<Decimal, Decimal> = BTreeMap::new();

        Self {
            exchange,
            symbol,
            bids,
            asks,
            last_update_id: u64::MIN
        }
    }

    fn bids(&self) -> &BTreeMap<Decimal, Decimal> {
        &self.bids
    }

    fn bids_mut(&mut self) -> &mut BTreeMap<Decimal, Decimal> {
        &mut self.bids
    }

    fn asks(&self) -> &BTreeMap<Decimal, Decimal> {
        &self.asks
    }

    fn asks_mut(&mut self) -> &mut BTreeMap<Decimal, Decimal> {
        &mut self.asks
    }
    
    pub fn add_bid(&mut self, level: [Decimal; 2]) -> Result<()> {
        let mut price = level[0];
        let quantity = level[1];

        let bids = self.bids_mut();
        bids.insert(price, quantity);

        
        Ok(())
    }

    pub fn add_ask(&mut self, level: [Decimal; 2]) -> Result<()> {
        let mut price = level[0];
        let quantity = level[1];

        let asks = self.asks_mut();
        asks.insert(price, quantity);

        Ok(())
    }
    pub fn get_bids_levels(&self) -> Result<Vec<Level>> {
        let bids = self.bids();
        
        let summary_bids = if bids.is_empty() {
            Vec::new()
        } else {
            let mut summary_bids = Vec::<Level>::with_capacity(10);
            for (&price, &amount) in bids.iter().take(10) {
                let level = Level {
                    exchange: self.exchange.to_string(),
                    price: price.to_f64().unwrap_or_default(),
                    amount: amount.to_f64().unwrap_or_default(),
                };
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
            for (&price, &amount) in asks.iter().take(10) {
                let level = Level {
                    exchange: self.exchange.to_string(),
                    price: price.to_f64().unwrap_or_default(),
                    amount: amount.to_f64().unwrap_or_default(),
                };
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
        // println!("e04");
        update.validate(self.last_update_id)?;
        // println!("==1");
        for (price, quantity) in update.bids_mut().iter() {
            self.add_bid([*price, *quantity])?;
        }
        for (price, quantity) in update.asks_mut().iter() {
            self.add_ask([*price, *quantity])?;
        }
        self.last_update_id = update.last_update_id();
        Ok(())
    }
}