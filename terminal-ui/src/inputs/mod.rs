use orderbook_merger::orderbook_summary::Summary;
use key::Key;

pub mod events;
pub mod key;

pub enum InputEvent {
    /// An input event occurred.
    Input(Key),
    /// An tick event occurred.
    Tick,
    Update(Summary),
}
