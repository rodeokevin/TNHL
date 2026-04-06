use serde::Deserialize;

use crate::models::{games::PlayerName, standings::TeamAbbrev};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameStoryReponse {
    pub summary: Option<Summary>,
}

impl GameStoryReponse {
    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub shootout: Vec<ShootoutAttempt>,
    pub team_game_stats: Vec<TeamGameStats>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShootoutAttempt {
    pub sequence: usize,
    pub player_id: u32,
    pub team_abbrev: TeamAbbrev,
    pub first_name: PlayerName,
    pub last_name: PlayerName,
    pub result: ShootoutAttemptResult,
    pub home_score: usize,
    pub away_score: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ShootoutAttemptResult {
    Save,
    Goal,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamGameStats {
    pub category: GameStatsCategory,
    pub away_value: StatValue,
    pub home_value: StatValue,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GameStatsCategory {
    Sog,
    FaceoffWinningPctg,
    PowerPlay,
    PowerPlayPctg,
    Pim,
    Hits,
    BlockedShots,
    Giveaways,
    Takeaways,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum StatValue {
    Int(u16),
    Float(f64),
    Str(String),
}
