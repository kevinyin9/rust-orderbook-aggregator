use anyhow::Result;

use self::actions::Actions;
use self::state::AppState;
use crate::inputs::key::Key;
use crate::app::actions::Action;
use orderbook_merger::orderbook_summary::Summary;

pub mod state;
pub mod actions;
pub mod ui;

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

/// The main application, containing the state
pub struct App {
    /// Contextual actions
    actions: Actions,
    /// State
    is_loading: bool,
    state: AppState,
}

impl App {
    pub fn new() -> Self {
        let actions = vec![Action::Quit].into();
        let is_loading = false;
        let state = AppState::default();

        Self {
            actions,
            is_loading,
            state,
        }
    }

    /// Handle a user action
    pub async fn do_action(&mut self, key: Key) -> AppReturn {
        if let Some(action) = self.actions.find(key) {
            match action {
                Action::Quit => AppReturn::Exit,
            }
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
