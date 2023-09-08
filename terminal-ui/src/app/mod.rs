use crate::inputs::key::Key;
use orderbook_merger::orderbook_summary::Summary;

pub mod ui;

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

/// The main application, containing the summary
pub struct App {
    summary: Summary,
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

    pub async fn press_key(&mut self, key: Key) -> AppReturn {
        if key == Key::Ctrl('c') {
            AppReturn::Exit
        } else {
            AppReturn::Continue
        }
    }

    pub async fn update_on_tick(&mut self) -> AppReturn {
        AppReturn::Continue
    }

    pub fn summary(&self) -> &Summary {
        &self.summary
    }

    pub async fn update_summary(&mut self, summary: Summary) -> AppReturn {
        self.summary = summary;
        AppReturn::Continue
    }
}
