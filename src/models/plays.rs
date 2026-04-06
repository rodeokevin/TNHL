use crate::models::games::PeriodDescriptor;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PlaysResponse {
    pub plays: Vec<PlayData>,
}

impl PlaysResponse {
    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayData {
    pub event_id: u32,
    pub period_descriptor: Option<PeriodDescriptor>,
    pub time_in_period: String,
    pub time_remaining: String,
    pub type_desc_key: TypeDescKey,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TypeDescKey {
    Hit,
    Stoppage,
    Faceoff,
    BlockedShot,
    Misconduct,
    MissedShot,
    ShotOnGoal,
    DelayedPenalty,
    Penalty,
    Giveaway,
    Takeaway,
    Goal,
    PeriodStart,
    PeriodEnd,
    #[serde(other)]
    Unknown,
}
