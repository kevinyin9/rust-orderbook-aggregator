pub mod exchange;
pub mod orderbook;
use serde::{Deserialize, Serialize};
use crate::orderbook::orderbook::OrderBookOnlyLevels;
use orderbook_summary::{Level, Summary};

pub mod orderbook_summary {
    tonic::include_proto!("orderbooksummary");
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Symbol {
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub enum ExchangeName {
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
    // println!("levels_count: {}", levels_count);
    let exchange_count = book_levels_vec.len();

    let mut bids = Vec::<Level>::with_capacity(levels_count);
    let mut asks = Vec::<Level>::with_capacity(levels_count);
    for i in 0..exchange_count {
        bids.append(&mut book_levels_vec[i].bids);
        asks.append(&mut book_levels_vec[i].asks);
    }
    // println!("bids lentgh: {}", bids.len());
    // println!("bids : {:?}", bids);
    bids.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
    asks.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());

    let take_bids = bids.into_iter().take(levels_count).collect::<Vec<Level>>();
    // println!("take_bids : {:?}", take_bids);
    let take_asks = asks.into_iter().take(levels_count).collect::<Vec<Level>>();
    let summary = Summary {
        spread: take_asks[0].price - take_bids[0].price,
        bids: take_bids,
        asks: take_asks,
    };
    summary
}