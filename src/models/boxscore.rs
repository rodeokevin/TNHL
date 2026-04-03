use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoxscoreResponse {
    pub player_by_game_stats: PlayerByGameStats,
}

impl BoxscoreResponse {
    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerByGameStats {
    pub away_team: Team,
    pub home_team: Team,
}

#[derive(Debug, Deserialize)]
pub struct Team {
    forwards: Vec<Skater>,
    defense: Vec<Skater>,
    goalies: Vec<Goalie>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skater {
    player_id: u32,
    sweater_number: u8,
    name: PlayerName,
    position: Position,
    goals: u8,
    assists: u8,
    points: u8,
    plus_minus: i8,
    pim: u8,
    hits: u8,
    power_play_goals: u8,
    sog: u8,
    faceoff_winning_pctg: Option<f32>,
    toi: String,
    blocked_shots: u8,
    shifts: u8,
    giveaways: u8,
    takeaways: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Goalie {
    plyer_id: u32,
    sweater_number: u8,
    name: PlayerName,
    position: Position,
    even_strength_shots_against: String,
    power_play_shots_against: String,
    shorthanded_shots_against: String,
    save_shots_against: String,
    save_pctg: f32,
    even_strength_goals_against: u8,
    power_play_goals_against: u8,
    shorthanded_goals_against: u8,
    goals_against: u8,
    toi: String,
    shots_against: u16,
    saves: u16,
}

#[derive(Debug, Deserialize)]
pub struct PlayerName {
    default: String,
}

#[derive(Debug, Deserialize)]
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