use std::{sync::Arc, io::stdout, collections::HashMap};
use config::Config;
use anyhow::Result;
use orderbook_merger::orderbook_summary::orderbook_aggregator_client::OrderbookAggregatorClient;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use terminal_ui::{App, ui, events::Events, InputEvent, key::Key};

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
        match events.next().await {
            InputEvent::Input(key) => {
                if key == Key::Ctrl('c') {
                    // exit
                    events.close();
                    break;
                }
            },
            InputEvent::Update(summary) => {
                app.summary = summary;
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
    let app = Arc::new(tokio::sync::Mutex::new(App::new()));
    let app_ui = Arc::clone(&app);

    start_ui(&app_ui).await?;

    Ok(())
}
