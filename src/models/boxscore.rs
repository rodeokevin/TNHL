use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoxscoreResponse {
    pub player_by_game_stats: Option<PlayerByGameStats>,
}

impl BoxscoreResponse {
    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerByGameStats {
    pub away_team: TeamData,
    pub home_team: TeamData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TeamData {
    pub forwards: Vec<Skater>,
    pub defense: Vec<Skater>,
    pub goalies: Vec<Goalie>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skater {
    pub player_id: u32,
    pub sweater_number: u8,
    pub name: PlayerName,
    pub position: Position,
    pub goals: u8,
    pub assists: u8,
    pub points: u8,
    pub plus_minus: i8,
    pub pim: u8,
    pub hits: u8,
    pub power_play_goals: u8,
    pub sog: u8,
    pub faceoff_winning_pctg: Option<f32>,
    pub toi: String,
    pub blocked_shots: u8,
    pub shifts: u8,
    pub giveaways: u8,
    pub takeaways: u8,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Goalie {
    pub player_id: u32,
    pub sweater_number: u8,
    pub pim: u8,
    pub name: PlayerName,
    pub position: Position,
    pub even_strength_shots_against: String,
    pub power_play_shots_against: String,
    pub shorthanded_shots_against: String,
    pub save_shots_against: String,
    pub save_pctg: Option<f32>,
    pub even_strength_goals_against: u8,
    pub power_play_goals_against: u8,
    pub shorthanded_goals_against: u8,
    pub goals_against: u8,
    pub toi: String,
    pub shots_against: u16,
    pub saves: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerName {
    pub default: String,
}

#[derive(Debug, Clone, Deserialize)]
pub enum Position {
    #[serde(rename = "L")]
    LeftWing,
    #[serde(rename = "R")]
    RightWing,
    #[serde(rename = "C")]
    Center,
    #[serde(rename = "D")]
    Defense,
    #[serde(rename = "G")]
    Goalie,
}
