use tokio::sync::mpsc::Sender;

use crate::models::{
    boxscore::BoxscoreResponse, game_story::GameStoryReponse, games::GamesResponse,
    standings::StandingsResponse,
};

pub mod boxscore;
pub mod game_story;
pub mod games;
pub mod standings;

/// Events sent to the main application loop.
#[derive(Debug)]
pub enum AppEvent {
    StandingsUpdate(StandingsResponse),
    GamesUpdate {
        game_ids: Vec<u32>,
        parsed_games: GamesResponse,
    },
    BoxscoreUpdate {
        game_id: u32,
        parsed_boxscore: BoxscoreResponse,
    },
    GameStoryUpdate {
        game_id: u32,
        parsed_game_story: GameStoryReponse,
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
