use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

use crate::models::playoffs::series::SeriesResponse;
use crate::sources::FetchInterval;
use crate::{AppEvent, Source};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TotalGoalsTarget {
    pub game_id: u32,
    pub season: String,
    pub letter: String,
}

pub enum TotalGoalsCommand {
    SetTargets(Vec<TotalGoalsTarget>),
    SetInterval(Duration),
}

pub struct TotalGoalsSource {
    client: reqwest::Client,
    rx: Receiver<TotalGoalsCommand>,
    targets: Vec<TotalGoalsTarget>,
    fetch_interval: Duration,
}

impl TotalGoalsSource {
    pub fn new(client: reqwest::Client, rx: Receiver<TotalGoalsCommand>) -> Self {
        Self {
            client,
            rx,
            targets: Vec::new(),
            fetch_interval: FetchInterval::InfoShortInterval.as_duration(),
        }
    }

    async fn fetch(&self, tx: &Sender<AppEvent>) {
        let fetches = self.targets.iter().map(|target| {
            let client = &self.client;
            async move {
                let letter = target.letter.to_lowercase();
                let url = format!(
                    "https://api-web.nhle.com/v1/schedule/playoff-series/{}/{}",
                    target.season, letter
                );

                match client.get(&url).send().await {
                    Ok(resp) => {
                        if let Ok(body) = resp.text().await {
                            match SeriesResponse::from_json(&body) {
                                Ok(parsed_series) => {
                                    let _ = tx
                                        .send(AppEvent::TotalGoalsUpdate {
                                            game_id: target.game_id,
                                            parsed_series,
                                        })
                                        .await;
                                }
                                Err(e) => log::error!(
                                    "Failed to parse total-goals series for game {}: {}",
                                    target.game_id,
                                    e
                                ),
                            }
                        }
                    }
                    Err(err) => log::warn!(
                        "Failed to fetch total-goals series for game {}: {}",
                        target.game_id,
                        err
                    ),
                }
            }
        });

        futures::future::join_all(fetches).await;
    }
}

#[async_trait::async_trait]
impl Source for TotalGoalsSource {
    async fn run(mut self: Box<Self>, tx: Sender<AppEvent>, cancel: CancellationToken) {
        let mut interval = tokio::time::interval(self.fetch_interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,

                Some(cmd) = self.rx.recv() => {
                    match cmd {
                        TotalGoalsCommand::SetTargets(mut targets) => {
                            targets.sort_by(|a, b| a.game_id.cmp(&b.game_id));
                            let mut current = self.targets.clone();
                            current.sort_by(|a, b| a.game_id.cmp(&b.game_id));
                            if targets != current {
                                log::debug!("Fetching Games total-goals series because targets changed");
                                self.targets = targets;
                                self.fetch(&tx).await;
                                interval.reset();
                            }
                        }
                        TotalGoalsCommand::SetInterval(new_interval) => {
                            if new_interval != self.fetch_interval {
                                log::debug!("Setting Games total-goals interval to {:?}", new_interval);
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
