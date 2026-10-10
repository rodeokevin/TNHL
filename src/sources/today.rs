use chrono::NaiveDate;
use tokio::sync::mpsc::Sender;
use tokio::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

use super::{AppEvent, Source, send_request};
use crate::models::games::score_now::ScoreNowResponse;

/// How often to check whether the NHL game day has changed
const POLL_INTERVAL: Duration = Duration::from_secs(300);
/// Background checks can wait longer than the startup one
const POLL_TIMEOUT: Duration = Duration::from_secs(10);

/// Keeps today's game day current while the app runs
pub struct TodaySource {
    client: reqwest::Client,
    /// The game day the app currently knows about
    today: NaiveDate,
}

impl TodaySource {
    pub fn new(client: reqwest::Client, today: NaiveDate) -> Self {
        Self { client, today }
    }
}

/// Fetch today's NHL game day. `timeout` bounds the whole request, so a slow
/// network can't hold up the caller.
pub async fn fetch_today(client: &reqwest::Client, timeout: Duration) -> Option<NaiveDate> {
    let url = "https://api-web.nhle.com/v1/score/now";
    match send_request(client.get(url).timeout(timeout)).await {
        Ok(resp) => match resp.text().await {
            Ok(body) => match ScoreNowResponse::from_json(&body) {
                Ok(parsed) => match NaiveDate::parse_from_str(&parsed.current_date, "%Y-%m-%d") {
                    Ok(date) => Some(date),
                    Err(e) => {
                        log::warn!("Could not parse score/now currentDate: {}", e);
                        None
                    }
                },
                Err(e) => {
                    log::warn!("Failed to parse score/now response: {}", e);
                    None
                }
            },
            Err(e) => {
                log::warn!("Failed to read score/now body: {}", e);
                None
            }
        },
        Err(e) => {
            log::warn!("Failed to fetch score/now: {}", e);
            None
        }
    }
}

#[async_trait::async_trait]
impl Source for TodaySource {
    async fn run(mut self: Box<Self>, tx: Sender<AppEvent>, cancel: CancellationToken) {
        // Today was already resolved at startup, so the first check waits a full interval
        let mut interval = tokio::time::interval_at(Instant::now() + POLL_INTERVAL, POLL_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,
                _ = interval.tick() => {
                    if let Some(date) = fetch_today(&self.client, POLL_TIMEOUT).await
                        && date != self.today
                    {
                        log::debug!("Game day changed from {} to {}", self.today, date);
                        self.today = date;
                        let _ = tx.send(AppEvent::TodayChanged { date }).await;
                    }
                }
            }
        }
    }
}
