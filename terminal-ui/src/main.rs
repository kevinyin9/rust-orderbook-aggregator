use std::{sync::Arc, io::stdout, collections::HashMap};
use config::Config;
use anyhow::Result;
use terminal_ui::app::{App, AppReturn};
use terminal_ui::inputs::{events::Events, InputEvent};
use terminal_ui::app::ui;
use orderbook_merger::orderbook_summary::orderbook_aggregator_client::OrderbookAggregatorClient;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

pub async fn start_ui(
    app: &Arc<tokio::sync::Mutex<App>>,
) -> Result<()> {
    // Configure Crossterm backend for tui
    let stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    let config = Config::builder()
        // read the setting.toml
        .add_source(config::File::with_name("orderbook-merger/src/setting"))
        .build()
        .unwrap()
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();

    let address = format!("https://{}:{}", config.get("server-ip").unwrap(), config.get("server-port").unwrap());

    let client: OrderbookAggregatorClient<tonic::transport::Channel> =
        OrderbookAggregatorClient::connect(address).await?;
    let mut events = Events::new(client);

    loop {
        let mut app = app.lock().await;

        // Render
        terminal.draw(|rect| ui::draw(rect, &app, 4))?;

        // Handle inputs
        let result = match events.next().await {
            InputEvent::Input(key) => app.press_key(key).await,
            InputEvent::Update(summary) => app.update_summary(summary).await,
        };
        // Check if we should exit
        if result == AppReturn::Exit {
            events.close();
            break;
        }
    }

    // Restore the terminal and close application
    terminal.clear()?;
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = Arc::new(tokio::sync::Mutex::new(App::new()));
    let app_ui = Arc::clone(&app);

    start_ui(&app_ui).await?;

    Ok(())
}
