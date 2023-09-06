use std::sync::Arc;
use std::io::stdout;

use anyhow::Result;
use terminal_ui::app::{App, AppReturn};
use terminal_ui::inputs::{events::Events, InputEvent};
use terminal_ui::app::ui;
use terminal_ui::io::{IoEvent, handler::IoAsyncHandler};
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

    let client: OrderbookAggregatorClient<tonic::transport::Channel> =
        OrderbookAggregatorClient::connect("http://127.0.0.1:5555").await?;
    // let decimals = summary_request.decimals;
    // let symbol = summary_request.symbol.clone();
    let mut events = Events::new(client);

    // Trigger state change from Init to Initialized
    {
        let mut app = app.lock().await;
        // Here we assume the the first load is a long task
        app.dispatch(IoEvent::Initialize).await;
    }

    loop {
        let mut app = app.lock().await;

        // Render
        terminal.draw(|rect| ui::draw(rect, &app, "BTCUSDT", 4))?;

        // Handle inputs
        let result = match events.next().await {
            InputEvent::Input(key) => app.do_action(key).await,
            InputEvent::Tick => app.update_on_tick().await,
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
    let (sync_io_tx, mut sync_io_rx) = tokio::sync::mpsc::channel::<IoEvent>(100);

    // We need to share the App between thread
    let app = Arc::new(tokio::sync::Mutex::new(App::new(sync_io_tx.clone())));
    let app_ui = Arc::clone(&app);

    // Handle IO in a specifc thread
    tokio::spawn(async move {
        let mut handler = IoAsyncHandler::new(app);
        while let Some(io_event) = sync_io_rx.recv().await {
            handler.handle_io_event(io_event).await;
        }
    });

    start_ui(&app_ui).await?;

    Ok(())
}
