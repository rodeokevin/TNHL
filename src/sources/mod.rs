use tokio::sync::mpsc::Sender;

pub mod standings;

/// Events sent to the main application loop.
#[derive(Debug)]
pub enum AppEvent {
    StandingsUpdate(String),
    Input(crossterm::event::KeyEvent),

    /// Periodic tick to refresh UI
    Tick,
}

#[async_trait::async_trait]
pub trait Source: Send + 'static {
    async fn run(
        self: Box<Self>,
        tx: Sender<AppEvent>,
        cancel: tokio_util::sync::CancellationToken,
    );
}
