use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GamesResponse {
    pub games: Vec<GameData>,
}

#[derive(Debug, Deserialize)]
pub struct GameData {
    pub id: u32,
}

impl GamesResponse {
    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
}
