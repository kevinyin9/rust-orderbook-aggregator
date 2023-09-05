pub mod exchanges;
pub mod orderbook;

use serde::{Deserialize, Serialize};
use crate::orderbook::orderbook::OrderBookOnlyLevels;
use orderbook_summary::{Level, Summary};
use anyhow::{ensure, Result};
use rust_decimal::Decimal;

pub mod orderbook_summary {
    tonic::include_proto!("orderbook_summary");
}

pub type DisplayAmount = Decimal;
impl ToStorage for DisplayAmount {
    fn to_storage(&self, scale: u32) -> Result<StorageAmount> {
        display_to_storage(*self, scale)
    }
}
pub type StorageAmount = u64;
impl ToDisplay for StorageAmount {
    fn to_display(&self, scale: u32) -> Result<DisplayAmount> {
        let mut display_price = Decimal::from(*self);
        display_price.set_scale(scale)?;
        Ok(display_price)
    }
}

pub fn display_to_storage(mut display_quantity: Decimal, scale: u32) -> Result<StorageAmount> {
    ensure!(
        display_quantity.is_sign_positive(),
        "quantity sign must be positive"
    );

    display_quantity = display_quantity.round_dp(scale);
    display_quantity.set_scale(0)?;

    let unpacked = display_quantity.unpack();
    ensure!(unpacked.hi == 0, "quantity is too large");

    let mut storage = unpacked.lo as u64;
    if unpacked.mid > 0 {
        storage += (unpacked.mid as u64) << 32;
    }
    Ok(storage)
}

pub trait ToDisplay {
    fn to_display(&self, scale: u32) -> Result<DisplayAmount>;
}
pub trait ToStorage {
    fn to_storage(&self, scale: u32) -> Result<StorageAmount>;
}


#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub enum Symbol {
    #[default]
    BTCUSDT,
    ETHUSDT
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Symbol::BTCUSDT => write!(f, "BTCUSDT"),
            Symbol::ETHUSDT => write!(f, "ETHUSDT"),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub enum ExchangeName {
    #[default]
    BINANCE,
    BITSTAMP,
}

impl std::fmt::Display for ExchangeName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ExchangeName::BINANCE => write!(f, "BINANCE"),
            ExchangeName::BITSTAMP => write!(f, "BITSTAMP"),
        }
    }
}

/// Returns a single summary of order book data aggregated from multiple exchanges
///
/// # Arguments
///
/// * `book_levels_vec` - A vector of [BookLevels] structs from multiple exchanges
/// * `symbol` - The symbol the order book data is for
// TODO: Remove symbol argument and get from BookLevels
pub fn make_summary(mut book_levels_vec: Vec<OrderBookOnlyLevels>, symbol: Symbol) -> Summary {
    let levels_count = book_levels_vec[0].bids.len();
    let exchange_count = book_levels_vec.len();

    let mut bids = Vec::<Level>::with_capacity(levels_count);
    let mut asks = Vec::<Level>::with_capacity(levels_count);
    for i in 0..exchange_count {
        bids.append(&mut book_levels_vec[i].bids);
        asks.append(&mut book_levels_vec[i].asks);
    }
    // println!("bids lentgh: {}", bids.len());
    
    // bids.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
    // asks.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
    // println!("bids : {:?}", bids);
    let take_bids = bids.into_iter().take(levels_count).collect::<Vec<Level>>();
    let take_asks = asks.into_iter().take(levels_count).collect::<Vec<Level>>();
    // println!("spread : {:?} {:?}", take_bids[0].price, take_asks[0].price);
    Summary {
        symbol: symbol.to_string(),
        spread: take_asks[0].price - take_bids[0].price,
        bids: take_bids,
        asks: take_asks,
    }
}