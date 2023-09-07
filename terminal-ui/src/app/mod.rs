use anyhow::Result;

use self::actions::Actions;
use self::state::AppState;
use crate::io::IoEvent;
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
    /// We could dispatch an IO event
    io_tx: tokio::sync::mpsc::Sender<IoEvent>,
    /// Contextual actions
    actions: Actions,
    /// State
    is_loading: bool,
    state: AppState,
}

impl App {
    pub fn new(io_tx: tokio::sync::mpsc::Sender<IoEvent>) -> Self {
        let actions = vec![Action::Quit].into();
        let is_loading = false;
        let state = AppState::default();

        Self {
            io_tx,
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

    /// Send a network event to the IO thread
    pub async fn dispatch(&mut self, action: IoEvent) {
        // `is_loading` will be set to false again after the async action has finished in io/handler.rs
        self.is_loading = true;
        if self.io_tx.send(action).await.is_err() {
            self.is_loading = false;
        };
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn is_loading(&self) -> bool {
        self.is_loading
    }

    pub async fn initialized(&mut self) -> Result<()> {
        self.state = AppState::initialized().await?;
        Ok(())
    }

    pub fn loaded(&mut self) {
        self.is_loading = false;
    }

    // pub fn slept(&mut self) {
    //     self.state.incr_sleep();
    // }

    pub async fn update_summary(&mut self, summary: Summary) -> AppReturn {
        self.state.update_summary(summary);
        AppReturn::Continue
    }
}
