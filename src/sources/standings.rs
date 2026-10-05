use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

use super::{AppEvent, Source};
use crate::models::standings::{DateResolution, SeasonBounds, resolve_date};
use crate::sources::{FetchInterval, StandingsResponse};

pub enum StandingsCommand {
    SetDate(String),
    SetInterval(Duration),
    SetSeasonBounds(Vec<SeasonBounds>),
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

    /// Resolve the requested date against the season bounds. Returns the
    /// classification, or `None` if bounds aren't available yet
    fn resolved(&self) -> Option<DateResolution> {
        let seasons = self.seasons.as_deref()?;
        let requested =
            chrono::NaiveDate::parse_from_str(&self.current_date, "%Y-%m-%d").ok()?;
        resolve_date(requested, seasons)
    }

    async fn fetch(&mut self, tx: &Sender<AppEvent>) {
        // Determine the date to fetch, or report that the requested date is
        // outside the available range\
        let date = match self.resolved() {
            Some(DateResolution::InSeason(d)) | Some(DateResolution::OffseasonGap(d)) => {
                d.format("%Y-%m-%d").to_string()
            }
            Some(DateResolution::AfterLatest(latest)) => {
                let msg = format!(
                    "No standings for this date yet. Latest available: {}.",
                    latest.format("%B %d, %Y")
                );
                log::debug!("Requested standings date is after latest season: {}", msg);
                let _ = tx
                    .send(AppEvent::StandingsOutOfRange { message: msg })
                    .await;
                return;
            }
            Some(DateResolution::BeforeEarliest(earliest)) => {
                let msg = format!(
                    "No standings for this date. Earliest available: {}.",
                    earliest.format("%B %d, %Y")
                );
                log::debug!("Requested standings date is before earliest season: {}", msg);
                let _ = tx
                    .send(AppEvent::StandingsOutOfRange { message: msg })
                    .await;
                return;
            }
            // Bounds unavailable or unparseable date: fetch the requested date
            None => self.current_date.clone(),
        };

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
                        StandingsCommand::SetSeasonBounds(seasons) => {
                            // Bounds are resolved exactly once by SeasonSource
                            self.seasons = Some(seasons);
                            self.fetch(&tx).await;
                            interval.reset();
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
        let start_year = NaiveDate::parse_from_str(start, "%Y-%m-%d")
            .unwrap()
            .format("%Y")
            .to_string()
            .parse::<u32>()
            .unwrap();
        SeasonBounds {
            id: start_year * 10000 + (start_year + 1),
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

    // These assert the date-classification the source relies on to decide
    // whether to fetch (InSeason/OffseasonGap) or report out-of-range
    // (AfterLatest/BeforeEarliest). The resolution logic itself lives in
    // `models::standings::resolve_date`.

    #[test]
    fn in_season_fetches_requested_date() {
        assert_eq!(
            resolve_date(d("2025-12-01"), &seasons()),
            Some(DateResolution::InSeason(d("2025-12-01")))
        );
    }

    #[test]
    fn offseason_gap_fetches_previous_season_end() {
        assert_eq!(
            resolve_date(d("2026-08-24"), &seasons()),
            Some(DateResolution::OffseasonGap(d("2026-04-17")))
        );
    }

    #[test]
    fn future_date_is_out_of_range() {
        assert_eq!(
            resolve_date(d("2300-10-01"), &seasons()),
            Some(DateResolution::AfterLatest(d("2027-04-10")))
        );
    }

    #[test]
    fn too_early_date_is_out_of_range() {
        assert_eq!(
            resolve_date(d("2000-01-01"), &seasons()),
            Some(DateResolution::BeforeEarliest(d("2024-10-04")))
        );
    }
}
