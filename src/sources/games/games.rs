use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

use crate::models::standings::{DateResolution, SeasonBounds, resolve_date};
use crate::sources::{FetchInterval, GamesResponse};
use crate::{AppEvent, Source};

pub enum GamesCommand {
    SetDate(String),
    SetInterval(Duration),
    SetSeasonBounds(Vec<SeasonBounds>),
}

pub struct GamesSource {
    client: reqwest::Client,
    rx: Receiver<GamesCommand>,
    current_date: String,
    fetch_interval: Duration,
    seasons: Option<Vec<SeasonBounds>>,
}
impl GamesSource {
    pub fn new(client: reqwest::Client, rx: Receiver<GamesCommand>, current_date: String) -> Self {
        Self {
            client,
            rx,
            current_date,
            fetch_interval: FetchInterval::GamesShortInterval.as_duration(),
            seasons: None,
        }
    }

    /// Classify the requested date against the season bounds. Returns `None` if
    /// bounds aren't available yet or the date can't be parsed
    fn resolved(&self) -> Option<DateResolution> {
        let seasons = self.seasons.as_deref()?;
        let requested =
            chrono::NaiveDate::parse_from_str(&self.current_date, "%Y-%m-%d").ok()?;
        resolve_date(requested, seasons)
    }

    async fn fetch(&mut self, tx: &Sender<AppEvent>) {
        match self.resolved() {
            Some(DateResolution::AfterLatest(latest)) => {
                let msg = format!(
                    "No games for this date yet. Latest available: {}.",
                    latest.format("%B %d, %Y")
                );
                log::debug!("Requested games date is after latest season: {}", msg);
                let _ = tx.send(AppEvent::GamesOutOfRange { message: msg }).await;
                return;
            }
            Some(DateResolution::BeforeEarliest(earliest)) => {
                let msg = format!(
                    "No games for this date. Earliest available: {}.",
                    earliest.format("%B %d, %Y")
                );
                log::debug!("Requested games date is before earliest season: {}", msg);
                let _ = tx.send(AppEvent::GamesOutOfRange { message: msg }).await;
                return;
            }
            _ => {}
        }

        let url = format!("https://api-web.nhle.com/v1/score/{}", self.current_date);

        match self.client.get(&url).send().await {
            Ok(resp) => {
                if let Ok(body) = resp.text().await {
                    match GamesResponse::from_json(&body) {
                        Ok(parsed_games) => {
                            let game_ids = parsed_games.games.iter().map(|g| g.id).collect();
                            let _ = tx
                                .send(AppEvent::GamesUpdate {
                                    game_ids,
                                    parsed_games,
                                })
                                .await;
                        }
                        Err(e) => log::error!("Failed to parse games: {}", e),
                    }
                }
            }
            Err(err) => log::warn!("Failed to fetch games: {}", err),
        }
    }
}

#[async_trait::async_trait]
impl Source for GamesSource {
    async fn run(mut self: Box<Self>, tx: Sender<AppEvent>, cancel: CancellationToken) {
        let mut interval = tokio::time::interval(self.fetch_interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,

                Some(cmd) = self.rx.recv() => {
                    match cmd {
                        GamesCommand::SetDate(date) => {
                            self.current_date = date;
                            self.fetch(&tx).await;
                            interval.reset();
                        },
                        GamesCommand::SetInterval(new_interval) => {
                            if new_interval != self.fetch_interval {
                                log::debug!("Setting games interval to {:?}", new_interval);
                                self.fetch_interval = new_interval;

                                interval = tokio::time::interval(self.fetch_interval);
                                interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                            }
                        }
                        GamesCommand::SetSeasonBounds(seasons) => {
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
