use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use serde::Deserialize;

use crate::models::games::games::{GameState, PeriodDescriptor, PeriodType, Venue};
use crate::models::{PlaceName, TeamAbbrev, TeamName};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesResponse {
    pub round: u8,
    pub round_abbrev: String,
    pub round_label: String,
    pub series_letter: String,
    pub needed_to_win: u8,
    pub length: u8,
    pub bottom_seed_team: SeedTeam,
    pub top_seed_team: SeedTeam,
    pub games: Vec<SeriesGame>,
}

impl SeriesResponse {
    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeedTeam {
    pub id: i32,
    pub name: TeamName,
    pub abbrev: TeamAbbrev,
    pub place_name: PlaceName,
    pub conference: Option<Conference>,
    pub record: String,
    pub series_wins: u8,
    pub division_abbrev: Option<String>,
    pub seed: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conference {
    pub name: String,
    pub abbrev: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesGame {
    pub id: u32,
    pub game_number: u8,
    pub if_necessary: bool,
    pub venue: Venue,
    #[serde(rename = "startTimeUTC")]
    pub start_time_utc: DateTime<Utc>,
    pub game_state: GameState,
    pub away_team: SeriesTeam,
    pub home_team: SeriesTeam,
    pub period_descriptor: Option<PeriodDescriptor>,
    pub series_status: Option<SeriesStatus>,
    pub game_outcome: Option<GameOutcome>,
}

impl SeriesGame {
    pub fn compute_local_time(&self, tz: Tz) -> DateTime<Tz> {
        self.start_time_utc.with_timezone(&tz)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SeriesTeam {
    pub id: u32,
    pub common_name: TeamName,
    pub place_name: PlaceName,
    pub abbrev: TeamAbbrev,
    pub score: Option<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesStatus {
    pub top_seed_wins: usize,
    pub bottom_seed_wins: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameOutcome {
    pub last_period_type: PeriodType,
    pub ot_periods: Option<u8>,
}
