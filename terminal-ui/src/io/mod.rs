use std::time::Duration;

use orderbook_merger::orderbook_summary::Summary;

pub mod handler;

#[derive(Debug, Clone)]
pub enum IoEvent {
    Initialize,      // Launch to initialize the application
    // Sleep(Duration), // Just take a little break
    // Update(Summary), // Update the summary
}
