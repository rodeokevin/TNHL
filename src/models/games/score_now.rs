use serde::Deserialize;

/// Minimal response shape for `GET /v1/score/now`.
///
/// The NHL API resolves the current "game day" server-side and returns it as
/// `currentDate` (format `YYYY-MM-DD`). This is the authoritative notion of
/// "today" for the schedule, accounting for late games that belong to the
/// previous calendar day (e.g. West-coast games finishing after midnight ET).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreNowResponse {
    pub current_date: String,
}

impl ScoreNowResponse {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
