use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

use super::{AppEvent, Source};
use crate::sources::{FetchInterval, GamesResponse};

pub enum GamesCommand {
    SetDate(String),
    SetInterval(Duration),
}

pub struct GamesSource {
    rx: Receiver<GamesCommand>,
    current_date: String,
    fetch_interval: Duration,
}
impl GamesSource {
    pub fn new(rx: Receiver<GamesCommand>, current_date: String) -> Self {
        Self {
            rx,
            current_date,
            fetch_interval: FetchInterval::GamesShortInterval.as_duration(),
        }
    }

    async fn fetch(&self, tx: &Sender<AppEvent>) {
        let url = format!("https://api-web.nhle.com/v1/score/{}", self.current_date);

        match reqwest::get(&url).await {
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
            Err(err) => log::info!("Failed to fetch games: {}", err),
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
                                log::info!("Setting games interval to {:?}", new_interval);
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
