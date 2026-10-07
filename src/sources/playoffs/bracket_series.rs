use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

use crate::models::playoffs::series::SeriesResponse;
use crate::sources::FetchInterval;
use crate::{AppEvent, Source};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BracketSeriesTarget {
    pub season: String,
    pub letter: String,
}

pub enum BracketSeriesCommand {
    SetTargets(Vec<BracketSeriesTarget>),
    SetInterval(Duration),
}

pub struct BracketSeriesSource {
    client: reqwest::Client,
    rx: Receiver<BracketSeriesCommand>,
    targets: Vec<BracketSeriesTarget>,
    fetch_interval: Duration,
}

impl BracketSeriesSource {
    pub fn new(client: reqwest::Client, rx: Receiver<BracketSeriesCommand>) -> Self {
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
                                        .send(AppEvent::BracketSeriesUpdate {
                                            letter: target.letter.clone(),
                                            parsed_series,
                                        })
                                        .await;
                                }
                                Err(e) => log::error!(
                                    "Failed to parse bracket series {}: {}",
                                    target.letter,
                                    e
                                ),
                            }
                        }
                    }
                    Err(err) => {
                        log::warn!("Failed to fetch bracket series {}: {}", target.letter, err)
                    }
                }
            }
        });

        futures::future::join_all(fetches).await;
    }
}

#[async_trait::async_trait]
impl Source for BracketSeriesSource {
    async fn run(mut self: Box<Self>, tx: Sender<AppEvent>, cancel: CancellationToken) {
        let mut interval = tokio::time::interval(self.fetch_interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,

                Some(cmd) = self.rx.recv() => {
                    match cmd {
                        BracketSeriesCommand::SetTargets(mut targets) => {
                            targets.sort_by(|a, b| {
                                (a.season.as_str(), a.letter.as_str())
                                    .cmp(&(b.season.as_str(), b.letter.as_str()))
                            });
                            let mut current = self.targets.clone();
                            current.sort_by(|a, b| {
                                (a.season.as_str(), a.letter.as_str())
                                    .cmp(&(b.season.as_str(), b.letter.as_str()))
                            });
                            if targets != current {
                                log::debug!("Fetching bracket series because targets changed");
                                self.targets = targets;
                                self.fetch(&tx).await;
                                interval.reset();
                            }
                        }
                        BracketSeriesCommand::SetInterval(new_interval) => {
                            if new_interval != self.fetch_interval {
                                log::debug!("Setting bracket series interval to {:?}", new_interval);
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
