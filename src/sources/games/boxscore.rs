use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

use crate::models::games::boxscore::BoxscoreResponse;
use crate::sources::FetchInterval;
use crate::{AppEvent, Source};

pub enum BoxscoreCommand {
    SetGameIds(Vec<u32>),
    SetInterval(Duration),
}

pub struct BoxscoreSource {
    client: reqwest::Client,
    rx: Receiver<BoxscoreCommand>,
    game_ids: Vec<u32>,
    fetch_interval: Duration,
}

impl BoxscoreSource {
    pub fn new(client: reqwest::Client, rx: Receiver<BoxscoreCommand>) -> Self {
        Self {
            client,
            rx,
            game_ids: Vec::new(),
            fetch_interval: FetchInterval::InfoShortInterval.as_duration(),
        }
    }

    async fn fetch(&self, tx: &Sender<AppEvent>) {
        // Fetch every game's boxscore concurrently so the UI isn't blocked on
        // one slow request; each result is sent as soon as it completes.
        let fetches = self.game_ids.iter().map(|&game_id| {
            let client = &self.client;
            async move {
                let url = format!(
                    "https://api-web.nhle.com/v1/gamecenter/{}/boxscore",
                    game_id
                );

                match client.get(&url).send().await {
                    Ok(resp) => {
                        if let Ok(body) = resp.text().await {
                            match BoxscoreResponse::from_json(&body) {
                                Ok(parsed_boxscore) => {
                                    let _ = tx
                                        .send(AppEvent::BoxscoreUpdate {
                                            game_id,
                                            parsed_boxscore,
                                        })
                                        .await;
                                }
                                Err(e) => log::error!(
                                    "Failed to parse boxscore for game id {}: {}",
                                    game_id,
                                    e
                                ),
                            }
                        }
                    }
                    Err(err) => {
                        log::warn!("Failed to fetch boxscore for game id {}: {}", game_id, err)
                    }
                }
            }
        });

        futures::future::join_all(fetches).await;
    }
}

#[async_trait::async_trait]
impl Source for BoxscoreSource {
    async fn run(mut self: Box<Self>, tx: Sender<AppEvent>, cancel: CancellationToken) {
        let mut interval = tokio::time::interval(self.fetch_interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,

                Some(cmd) = self.rx.recv() => {
                    match cmd {
                        BoxscoreCommand::SetGameIds(mut ids) => {
                            ids.sort();
                            let mut current = self.game_ids.clone();
                            current.sort();
                            // Only fetch if game ids changed since this command is called on every GamesUpdate event
                            if ids != current {
                                log::debug!("Fetching boxscore because game ids changed");
                                self.game_ids = ids;
                                self.fetch(&tx).await;
                                interval.reset();
                            }
                        },
                        BoxscoreCommand::SetInterval(new_interval) => {
                            if new_interval != self.fetch_interval {
                                log::debug!("Setting boxscore interval to {:?}", new_interval);
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
