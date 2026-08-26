use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

use crate::models::games::play_by_play::PlaysResponse;
use crate::sources::FetchInterval;
use crate::{AppEvent, Source};

pub enum PlaysCommand {
    SetGameIds(Vec<u32>),
    SetInterval(Duration),
}

pub struct PlaysSource {
    client: reqwest::Client,
    rx: Receiver<PlaysCommand>,
    game_ids: Vec<u32>,
    fetch_interval: Duration,
}

impl PlaysSource {
    pub fn new(client: reqwest::Client, rx: Receiver<PlaysCommand>) -> Self {
        Self {
            client,
            rx,
            game_ids: Vec::new(),
            fetch_interval: FetchInterval::GamesShortInterval.as_duration(),
        }
    }

    async fn fetch(&self, tx: &Sender<AppEvent>) {
        let fetches = self.game_ids.iter().map(|&game_id| {
            let client = &self.client;
            async move {
                let url = format!(
                    "https://api-web.nhle.com/v1/gamecenter/{}/play-by-play",
                    game_id
                );

                match client.get(&url).send().await {
                    Ok(resp) => {
                        if let Ok(body) = resp.text().await {
                            match PlaysResponse::from_json(&body) {
                                Ok(parsed_plays) => {
                                    let _ = tx
                                        .send(AppEvent::PlaysUpdate {
                                            game_id,
                                            parsed_plays,
                                        })
                                        .await;
                                }
                                Err(e) => log::error!(
                                    "Failed to parse plays for game id {}: {}",
                                    game_id,
                                    e
                                ),
                            }
                        }
                    }
                    Err(err) => {
                        log::warn!("Failed to fetch plays for game id {}: {}", game_id, err)
                    }
                }
            }
        });

        futures::future::join_all(fetches).await;
    }
}

#[async_trait::async_trait]
impl Source for PlaysSource {
    async fn run(mut self: Box<Self>, tx: Sender<AppEvent>, cancel: CancellationToken) {
        let mut interval = tokio::time::interval(self.fetch_interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,

                Some(cmd) = self.rx.recv() => {
                    match cmd {
                        PlaysCommand::SetGameIds(mut ids) => {
                            ids.sort();
                            let mut current = self.game_ids.clone();
                            current.sort();
                            if ids != current {
                                log::debug!("Fetching plays because game ids changed");
                                self.game_ids = ids;
                                self.fetch(&tx).await;
                                interval.reset();
                            }
                        },
                        PlaysCommand::SetInterval(new_interval) => {
                            if new_interval != self.fetch_interval {
                                log::debug!("Setting plays interval to {:?}", new_interval);
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
