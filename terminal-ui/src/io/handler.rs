use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use orderbook_merger::orderbook_summary::Summary;

use super::IoEvent;
use crate::app::App;

/// In the IO thread, we handle IO event without blocking the UI thread
pub struct IoAsyncHandler {
    app: Arc<tokio::sync::Mutex<App>>,
}

impl IoAsyncHandler {
    pub fn new(app: Arc<tokio::sync::Mutex<App>>) -> Self {
        Self { app }
    }

    /// We could be async here
    pub async fn handle_io_event(&mut self, io_event: IoEvent) {
        let result = match io_event {
            IoEvent::Initialize => self.do_initialize().await,
            IoEvent::Sleep(duration) => self.do_sleep(duration).await,
            IoEvent::Update(summary) => self.do_update(summary).await,
        };

        if let Err(err) = result {
            println!("Error: {:?}", err);
        }

        let mut app = self.app.lock().await;
        app.loaded();
    }

    /// We use dummy implementation here, just wait 1s
    async fn do_initialize(&mut self) -> Result<()> {
        let mut app = self.app.lock().await;
        // tokio::time::sleep(Duration::from_secs(1)).await;
        app.initialized().await?; // we could update the app state

        Ok(())
    }

    /// Just take a little break
    async fn do_sleep(&mut self, duration: Duration) -> Result<()> {
        tokio::time::sleep(duration).await;
        // Notify the app for having slept
        let mut app = self.app.lock().await;
        app.slept();

        Ok(())
    }

    async fn do_update(&mut self, summary: Summary) -> Result<()> {
        let mut app = self.app.lock().await;
        app.update_summary(summary).await;

        Ok(())
    }
}
