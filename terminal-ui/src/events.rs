use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use orderbook_merger::orderbook_summary::{orderbook_aggregator_client::OrderbookAggregatorClient, Empty};
use tokio_stream::StreamExt;

use super::{Key, InputEvent};

// The `Events` struct represents event channels for receiving and sending input and Update events, with
// a mechanism to stop event.
// Each event type is handled in its own thread and returned to a common `Receiver`.
pub struct Events {
    rx: tokio::sync::mpsc::Receiver<InputEvent>,
    _tx: tokio::sync::mpsc::Sender<InputEvent>,
    stop_capture: Arc<AtomicBool>, // Atomic boolean value that can be shared across multiple threads safely.
}

impl Events {
    // Constructs an new instance of `Events` with the default config.
    pub fn new(
        mut client: OrderbookAggregatorClient<tonic::transport::Channel>,
    ) -> Events {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let stop_capture = Arc::new(AtomicBool::new(false));

        let event_tx = tx.clone();
        let client_tx = tx.clone();
        let event_stop_capture = stop_capture.clone();

        tokio::spawn(async move {
            // Receiving order book summaries from the `client` and
            // sending them as `InputEvent::Update` through the `client_tx` channel.
            let request = tonic::Request::new(Empty {});
            let mut stream = client.book_summary(request).await.unwrap().into_inner();
            while let Some(summary) = stream.next().await {
                match summary {
                    Ok(summary) => {
                        if let Err(err) = client_tx.send(InputEvent::Update(summary)).await {
                            println!("Error!, {}", err);
                        }
                    }
                    Err(err) => {
                        println!("Error!, {}", err);
                    }
                };
            }
        });

        tokio::spawn(async move {
            // Continuously checks for keyboard events.
            loop {
                if crossterm::event::poll(Duration::from_millis(100)).unwrap() {
                    if let crossterm::event::Event::Key(key) = crossterm::event::read().unwrap() {
                        let key = Key::from(key);
                        if let Err(err) = event_tx.send(InputEvent::Input(key)).await {
                            println!("Oops!, {}", err);
                        }
                    }
                }

                if event_stop_capture.load(Ordering::Relaxed) {
                    break;
                }
            }
        });

        Events {
            rx,
            _tx: tx,
            stop_capture,
        }
    }

    // Attempts to read an event.
    pub async fn next(&mut self) -> InputEvent {
        self.rx.recv().await.unwrap()
    }

    // Close
    pub fn close(&mut self) {
        self.stop_capture.store(true, Ordering::Relaxed)
    }
}
