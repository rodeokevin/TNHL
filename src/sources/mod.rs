use std::time::Duration;
use tokio::sync::mpsc::Sender;

use crate::models::{
    games::{boxscore::BoxscoreResponse, game_story::GameStoryResponse, games::GamesResponse},
    playoffs::{bracket::BracketResponse, series::SeriesResponse},
    standings::{SeasonBounds, StandingsResponse},
    team_stats::TeamStatsResponse,
};

pub mod games;
pub mod playoffs;
pub mod season;
pub mod standings;
pub mod teams_stats;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FetchInterval {
    GamesShortInterval,
    GamesLongInterval,
    InfoShortInterval,
    InfoLongInterval,
}
impl FetchInterval {
    pub fn as_duration(&self) -> Duration {
        match self {
            FetchInterval::GamesShortInterval => Duration::from_secs(10),
            FetchInterval::GamesLongInterval => Duration::from_secs(60),
            FetchInterval::InfoShortInterval => Duration::from_secs(30),
            FetchInterval::InfoLongInterval => Duration::from_secs(600),
        }
    }
}

/// Events sent to the main application loop.
#[derive(Debug)]
pub enum AppEvent {
    /// The current NHL season (end year) resolved at startup from season bounds.
    SeasonResolved {
        year: i32,
    },
    StandingsUpdate {
        standings: StandingsResponse,
        season: Option<SeasonBounds>,
    },
    TeamStatsRegularSeasonUpdate(TeamStatsResponse),
    TeamStatsPlayoffsUpdate(TeamStatsResponse),
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
        parsed_game_story: GameStoryResponse,
    },
    BracketUpdate(BracketResponse),
    SeriesUpdate(SeriesResponse),
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
