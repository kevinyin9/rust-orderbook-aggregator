use std::{sync::Arc, io::stdout, collections::HashMap};
use config::Config;
use anyhow::Result;
use orderbook_merger::orderbook_summary::orderbook_aggregator_client::OrderbookAggregatorClient;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use orderbook_merger::orderbook_summary::Summary;
use terminal_ui::{ui, events::Events, InputEvent, Key};

pub async fn start_ui() -> Result<()> {
    // Configure Crossterm backend for tui
    let stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    let config = Config::builder()
        // read the setting.toml
        .add_source(config::File::with_name("orderbook-merger/src/setting"))
        .build()
        .unwrap()
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();

    let address = format!("https://{}:{}", config["server-ip"], config["server-port"]);

    let client = OrderbookAggregatorClient::connect(address).await?;
    let mut events = Events::new(client);
    let summary = Arc::new(tokio::sync::Mutex::new(Summary {
        spread: 0.0, 
        bids: Vec::new(),
        asks: Vec::new(),
    }));

    loop {
        let mut summary = summary.lock().await;
        // Render
        terminal.draw(|rect| ui::draw(rect, &summary, 4))?;

        // Handle inputs
        match events.next().await {
            InputEvent::Input(key) => {
                if key == Key::Ctrl('c') {
                    // exit
                    events.close();
                    break;
                }
            },
            InputEvent::Update(new_summary) => {
                *summary = new_summary;
            },
        };
    }

    // Restore the terminal and close application
    terminal.clear()?;
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    start_ui().await?;
    Ok(())
}
