use serde::Deserialize;

use crate::models::{TeamAbbrevWrapper, TeamName};

#[derive(Debug, Deserialize)]
pub struct StandingsResponse {
    pub standings: Vec<TeamData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamData {
    pub team_name: TeamName,
    pub team_abbrev: TeamAbbrevWrapper,
    pub season_id: u32,
    pub clinch_indicator: Option<String>,
    pub conference_abbrev: String,
    pub division_abbrev: String,
    pub conference_sequence: u8,
    pub wildcard_sequence: u8,
    pub division_sequence: u8,
    pub league_sequence: u8,
    pub games_played: u16,
    pub wins: u8,
    pub losses: u8,
    pub ot_losses: u8,
    pub points: u16,
    pub point_pctg: f64,
    pub regulation_wins: u8,
    pub regulation_plus_ot_wins: u8,
    pub goal_for: u16,
    pub goal_against: u16,
    pub home_wins: u8,
    pub home_ot_losses: u8,
    pub home_losses: u8,
    pub road_wins: u8,
    pub road_ot_losses: u8,
    pub road_losses: u8,
    pub shootout_wins: u8,
    pub shootout_losses: u8,
    pub l10_wins: u8,
    pub l10_ot_losses: u8,
    pub l10_losses: u8,
    pub streak_code: String,
    pub streak_count: u8,
}

impl StandingsResponse {
    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
}

#[derive(Debug, Deserialize)]
pub struct StandingsSeasonResponse {
    pub seasons: Vec<SeasonBounds>,
}

/// The date range a season's standings are valid for.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeasonBounds {
    pub standings_start: String,
    pub standings_end: String,
}

impl StandingsSeasonResponse {
    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
}

impl SeasonBounds {
    /// Parsed start date, if valid.
    pub fn start(&self) -> Option<chrono::NaiveDate> {
        chrono::NaiveDate::parse_from_str(&self.standings_start, "%Y-%m-%d").ok()
    }
    /// Parsed end date, if valid.
    pub fn end(&self) -> Option<chrono::NaiveDate> {
        chrono::NaiveDate::parse_from_str(&self.standings_end, "%Y-%m-%d").ok()
    }
}

/// Resolve the NHL "season end year" for a given date from the season bounds.
///
/// This is the year used to build season identifiers (e.g. `20262027`) for
/// team stats and playoffs:
/// - if `date` is within a season, that season's end year;
/// - if `date` is in an offseason gap or after the latest season, the most
///   recent completed season's end year
/// - if `date` is before the earliest season, the earliest season's end year.
///
/// Returns `None` only if the season list is empty or has unparseable dates.
pub fn season_end_year_for_date(date: chrono::NaiveDate, seasons: &[SeasonBounds]) -> Option<i32> {
    use chrono::Datelike;

    // Season containing the date.
    if let Some(s) = seasons.iter().find(|s| match (s.start(), s.end()) {
        (Some(start), Some(end)) => date >= start && date <= end,
        _ => false,
    }) {
        return s.end().map(|e| e.year());
    }

    // Otherwise the most recent season that ended before the date.
    if let Some(end) = seasons
        .iter()
        .filter_map(|s| s.end())
        .filter(|&end| end < date)
        .max()
    {
        return Some(end.year());
    }

    // Otherwise (date before any season) the earliest season's end year.
    seasons
        .iter()
        .filter_map(|s| s.end())
        .min()
        .map(|e| e.year())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn season(start: &str, end: &str) -> SeasonBounds {
        SeasonBounds {
            standings_start: start.to_string(),
            standings_end: end.to_string(),
        }
    }

    fn seasons() -> Vec<SeasonBounds> {
        vec![
            season("2024-10-04", "2025-04-17"),
            season("2025-10-07", "2026-04-17"),
            season("2026-09-29", "2027-04-10"),
        ]
    }

    #[test]
    fn end_year_within_season() {
        // Oct 29 2026 is within the 2026-2027 season -> end year 2027.
        assert_eq!(
            season_end_year_for_date(d("2026-10-29"), &seasons()),
            Some(2027)
        );
        // Mid 2025-2026 season.
        assert_eq!(
            season_end_year_for_date(d("2025-12-01"), &seasons()),
            Some(2026)
        );
    }

    #[test]
    fn end_year_offseason_uses_previous_season() {
        // Aug 24 2026 is between 2025-26 and 2026-27 -> previous season end 2026.
        assert_eq!(
            season_end_year_for_date(d("2026-08-24"), &seasons()),
            Some(2026)
        );
    }

    #[test]
    fn end_year_after_latest_and_before_first() {
        assert_eq!(
            season_end_year_for_date(d("2030-01-01"), &seasons()),
            Some(2027)
        );
        assert_eq!(
            season_end_year_for_date(d("2000-01-01"), &seasons()),
            Some(2025)
        );
    }

    #[test]
    fn end_year_empty_is_none() {
        assert_eq!(season_end_year_for_date(d("2026-10-29"), &[]), None);
    }
}
