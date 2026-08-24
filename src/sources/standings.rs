use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

use super::{AppEvent, Source};
use crate::models::standings::{SeasonBounds, StandingsSeasonResponse};
use crate::sources::{FetchInterval, StandingsResponse};

pub enum StandingsCommand {
    SetDate(String),
    SetInterval(Duration),
}

pub struct StandingsSource {
    client: reqwest::Client,
    rx: Receiver<StandingsCommand>,
    current_date: String,
    fetch_interval: Duration,
    seasons: Option<Vec<SeasonBounds>>,
}
impl StandingsSource {
    pub fn new(
        client: reqwest::Client,
        rx: Receiver<StandingsCommand>,
        current_date: String,
    ) -> Self {
        Self {
            client,
            rx,
            current_date,
            fetch_interval: FetchInterval::InfoShortInterval.as_duration(),
            seasons: None,
        }
    }

    /// Fetch and cache the season bounds if we don't have them yet. Left as
    /// `None` on failure so it is retried on the next fetch.
    async fn ensure_seasons(&mut self) {
        if self.seasons.is_some() {
            return;
        }
        let url = "https://api-web.nhle.com/v1/standings-season";
        match self.client.get(url).send().await {
            Ok(resp) => match resp.text().await {
                Ok(body) => match StandingsSeasonResponse::from_json(&body) {
                    Ok(parsed) if !parsed.seasons.is_empty() => {
                        log::debug!("Fetched {} standings seasons", parsed.seasons.len());
                        self.seasons = Some(parsed.seasons);
                    }
                    Ok(_) => log::warn!("standings-season returned no seasons; will retry"),
                    Err(e) => log::error!("Failed to parse standings-season: {}", e),
                },
                Err(e) => log::warn!("Failed to read standings-season body: {}", e),
            },
            Err(e) => log::warn!("Failed to fetch standings-season: {}", e),
        }
    }

    /// The date actually used to fetch standings: the requested date capped to
    /// the appropriate season. Falls back to the requested date if the season
    /// bounds aren't available or the date can't be parsed.
    fn effective_date(&self) -> String {
        let Some(seasons) = self.seasons.as_deref() else {
            return self.current_date.clone();
        };
        let Ok(requested) = chrono::NaiveDate::parse_from_str(&self.current_date, "%Y-%m-%d")
        else {
            return self.current_date.clone();
        };
        match cap_date(requested, seasons) {
            Some(capped) => capped.format("%Y-%m-%d").to_string(),
            None => self.current_date.clone(),
        }
    }

    async fn fetch(&mut self, tx: &Sender<AppEvent>) {
        // Make sure we have the season bounds so we can cap the date.
        self.ensure_seasons().await;

        let date = self.effective_date();
        let season = self.matched_season(&date);
        let url = format!("https://api-web.nhle.com/v1/standings/{}", date);

        match self.client.get(&url).send().await {
            Ok(resp) => {
                if let Ok(body) = resp.text().await {
                    // Parse the JSON
                    match StandingsResponse::from_json(&body) {
                        Ok(parsed_standings) => {
                            log::debug!("Standings data successfully parsed (date {})", date);
                            let _ = tx
                                .send(AppEvent::StandingsUpdate {
                                    standings: parsed_standings,
                                    season,
                                })
                                .await;
                            log::debug!("Sent standings data to app");
                        }
                        Err(e) => log::error!("Failed to parse standings: {}", e),
                    }
                }
            }
            Err(err) => log::warn!("Failed to fetch standings: {}", err),
        }
    }

    /// The season bounds whose range contains `date` (the effective fetch date).
    fn matched_season(&self, date: &str) -> Option<SeasonBounds> {
        let seasons = self.seasons.as_deref()?;
        let d = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
        seasons
            .iter()
            .find(|s| match (s.start(), s.end()) {
                (Some(start), Some(end)) => d >= start && d <= end,
                _ => false,
            })
            .cloned()
    }
}

/// Cap a requested date to the season whose standings should be shown.
/// Returns `None` only if the season list is empty.
fn cap_date(requested: chrono::NaiveDate, seasons: &[SeasonBounds]) -> Option<chrono::NaiveDate> {
    // In-season: use the requested date as-is.
    let in_season = seasons.iter().any(|s| match (s.start(), s.end()) {
        (Some(start), Some(end)) => requested >= start && requested <= end,
        _ => false,
    });
    if in_season {
        return Some(requested);
    }

    // Otherwise, cap to the latest season end that is before the requested date.
    let latest_prior_end = seasons
        .iter()
        .filter_map(|s| s.end())
        .filter(|&end| end < requested)
        .max();
    if let Some(end) = latest_prior_end {
        return Some(end);
    }

    // Requested date is before any season: use the earliest start.
    seasons.iter().filter_map(|s| s.start()).min()
}

#[async_trait::async_trait]
impl Source for StandingsSource {
    async fn run(mut self: Box<Self>, tx: Sender<AppEvent>, cancel: CancellationToken) {
        let mut interval = tokio::time::interval(self.fetch_interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,
                Some(cmd) = self.rx.recv() => {
                    match cmd {
                        StandingsCommand::SetDate(date) => {
                            self.current_date = date;
                            self.fetch(&tx).await;
                            interval.reset();
                        }
                        StandingsCommand::SetInterval(new_interval) => {
                            if new_interval != self.fetch_interval {
                                log::debug!("Setting standings interval to {:?}", new_interval);
                                self.fetch_interval = new_interval;

                                interval = tokio::time::interval(self.fetch_interval);
                                interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                            }
                        }
                    }
                },
                _ = interval.tick() => {
                    self.fetch(&tx).await;
                }
            }
        }
    }
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
    fn in_season_date_is_unchanged() {
        // A date within a season's range is returned as-is.
        assert_eq!(cap_date(d("2025-12-01"), &seasons()), Some(d("2025-12-01")));
        // Boundary dates are inclusive.
        assert_eq!(cap_date(d("2025-10-07"), &seasons()), Some(d("2025-10-07")));
        assert_eq!(cap_date(d("2026-04-17"), &seasons()), Some(d("2026-04-17")));
    }

    #[test]
    fn offseason_gap_caps_to_previous_season_end() {
        // 2026-08-24 is between 2025-26 (ends 04-17) and 2026-27 (starts 09-29):
        // cap to the previous season's end.
        assert_eq!(cap_date(d("2026-08-24"), &seasons()), Some(d("2026-04-17")));
        assert_eq!(cap_date(d("2025-06-01"), &seasons()), Some(d("2025-04-17")));
    }

    #[test]
    fn after_latest_season_caps_to_latest_end() {
        assert_eq!(cap_date(d("2030-01-01"), &seasons()), Some(d("2027-04-10")));
    }

    #[test]
    fn before_first_season_uses_earliest_start() {
        assert_eq!(cap_date(d("2000-01-01"), &seasons()), Some(d("2024-10-04")));
    }

    #[test]
    fn empty_seasons_returns_none() {
        assert_eq!(cap_date(d("2026-08-24"), &[]), None);
    }
}
