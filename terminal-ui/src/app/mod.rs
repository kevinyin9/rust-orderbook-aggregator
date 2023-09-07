use anyhow::Result;

use self::state::AppState;
use crate::inputs::key::Key;
use orderbook_merger::orderbook_summary::Summary;

pub mod state;
pub mod ui;

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

/// The main application, containing the state
pub struct App {
    /// State
    is_loading: bool,
    state: AppState,
}

impl App {
    pub fn new() -> Self {
        let is_loading = false;
        let state = AppState::default();

        Self {
            is_loading,
            state,
        }
    }

    pub async fn press_key(&mut self, key: Key) -> AppReturn {
        if key == Key::Ctrl('c') {
            AppReturn::Exit
        } else {
            AppReturn::Continue
        }
    }

    /// We could update the app or dispatch event on tick
    pub async fn update_on_tick(&mut self) -> AppReturn {
        // here we just increment a counter
        AppReturn::Continue
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub async fn initialized(&mut self) -> Result<()> {
        self.state = AppState::initialized().await?;
        Ok(())
    }

    pub async fn update_summary(&mut self, summary: Summary) -> AppReturn {
        self.state.update_summary(summary);
        AppReturn::Continue
    }
}
