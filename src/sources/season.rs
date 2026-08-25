use chrono::{Datelike, NaiveDate};
use tokio::sync::mpsc::Sender;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

use super::{AppEvent, Source};
use crate::models::standings::{StandingsSeasonResponse, season_end_year_for_date};

/// How long to wait between retries if the season bounds can't be fetched.
const RETRY_INTERVAL: Duration = Duration::from_secs(30);

/// Resolves the current NHL season (end year) from the `/standings-season`
/// bounds so team stats and playoffs start on the correct season (handling the
/// offseason). Emits `AppEvent::SeasonResolved` once resolved from the API,
/// then stops. If the bounds can't be fetched it emits a date-based heuristic
/// fallback so those pages aren't left empty, and keeps retrying the real
/// bounds until it succeeds.
pub struct SeasonSource {
    client: reqwest::Client,
    today: NaiveDate,
}

impl SeasonSource {
    pub fn new(client: reqwest::Client, today: NaiveDate) -> Self {
        Self { client, today }
    }

    /// Try to resolve the season year from the API. Returns the year on success.
    async fn try_resolve(&self) -> Option<i32> {
        let url = "https://api-web.nhle.com/v1/standings-season";
        match self.client.get(url).send().await {
            Ok(resp) => match resp.text().await {
                Ok(body) => match StandingsSeasonResponse::from_json(&body) {
                    Ok(parsed) => match season_end_year_for_date(self.today, &parsed.seasons) {
                        Some(year) => Some(year),
                        None => {
                            log::warn!("Could not resolve season year (no seasons)");
                            None
                        }
                    },
                    Err(e) => {
                        log::error!("Failed to parse standings-season: {}", e);
                        None
                    }
                },
                Err(e) => {
                    log::warn!("Failed to read standings-season body: {}", e);
                    None
                }
            },
            Err(e) => {
                log::warn!("Failed to fetch standings-season: {}", e);
                None
            }
        }
    }

    /// Fallback season end year when the bounds are unavailable: the previous
    /// behavior of using today's calendar year.
    fn fallback_year(&self) -> i32 {
        self.today.year()
    }
}

#[async_trait::async_trait]
impl Source for SeasonSource {
    async fn run(self: Box<Self>, tx: Sender<AppEvent>, cancel: CancellationToken) {
        // First attempt.
        if let Some(year) = tokio::select! {
            _ = cancel.cancelled() => return,
            year = self.try_resolve() => year,
        } {
            log::debug!("Resolved current season end year: {}", year);
            let _ = tx.send(AppEvent::SeasonResolved { year }).await;
            return;
        }

        // Fetch failed: fall back to today's calendar year (the previous
        // behavior) so the pages aren't empty, then keep retrying the real
        // bounds until one succeeds.
        let fallback = self.fallback_year();
        log::warn!(
            "Using fallback season end year {} while retrying season bounds",
            fallback
        );
        let _ = tx.send(AppEvent::SeasonResolved { year: fallback }).await;

        let mut interval = tokio::time::interval(RETRY_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        interval.tick().await; // consume the immediate first tick
        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,
                _ = interval.tick() => {
                    if let Some(year) = self.try_resolve().await {
                        log::debug!("Resolved season end year on retry: {}", year);
                        let _ = tx.send(AppEvent::SeasonResolved { year }).await;
                        break;
                    }
                }
            }
        }
    }
}
