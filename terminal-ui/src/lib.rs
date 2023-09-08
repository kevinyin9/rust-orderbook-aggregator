pub mod ui;
pub mod events;
pub mod key;

use orderbook_merger::orderbook_summary::Summary;

pub enum InputEvent {
    /// An input event occurred.
    Input(key::Key),
    Update(Summary),
}

/// The main application, containing the summary
pub struct App {
    pub summary: Summary,
}

impl App {
    pub fn new() -> Self {
        Self {
            summary: Summary {
                spread: 0.0, 
                bids: Vec::new(),
                asks: Vec::new(),
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}