pub mod exchange;
pub mod orderbook;
use serde::{Deserialize, Serialize};

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

