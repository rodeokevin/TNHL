use tokio::sync::mpsc::Sender;

pub mod boxscore;
pub mod games;
pub mod standings;

/// Events sent to the main application loop.
#[derive(Debug)]
pub enum AppEvent {
    StandingsUpdate(String),
    GamesUpdate(String),
    BoxscoreUpdate {
        game_id: u32,
        data: String,
    },
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
