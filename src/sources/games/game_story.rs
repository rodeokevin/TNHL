use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

use crate::models::games::game_story::GameStoryResponse;
use crate::sources::FetchInterval;
use crate::{AppEvent, Source};

pub enum GameStoryCommand {
    SetGameIds(Vec<u32>),
    SetInterval(Duration),
}

pub struct GameStorySource {
    client: reqwest::Client,
    rx: Receiver<GameStoryCommand>,
    game_ids: Vec<u32>,
    fetch_interval: Duration,
}

impl GameStorySource {
    pub fn new(client: reqwest::Client, rx: Receiver<GameStoryCommand>) -> Self {
        Self {
            client,
            rx,
            game_ids: Vec::new(),
            fetch_interval: FetchInterval::InfoShortInterval.as_duration(),
        }
    }

    async fn fetch(&self, tx: &Sender<AppEvent>) {
        // Fetch every game's story concurrently so the UI isn't blocked on one
        // slow request; each result is sent as soon as it completes.
        let fetches = self.game_ids.iter().map(|&game_id| {
            let client = &self.client;
            async move {
                let url = format!("https://api-web.nhle.com/v1/wsc/game-story/{}", game_id);

                match client.get(&url).send().await {
                    Ok(resp) => {
                        if let Ok(body) = resp.text().await {
                            match GameStoryResponse::from_json(&body) {
                                Ok(parsed_game_story) => {
                                    let _ = tx
                                        .send(AppEvent::GameStoryUpdate {
                                            game_id,
                                            parsed_game_story,
                                        })
                                        .await;
                                }
                                Err(e) => log::error!(
                                    "Failed to parse game story for game id {}: {}",
                                    game_id,
                                    e
                                ),
                            }
                        }
                    }
                    Err(err) => {
                        log::info!(
                            "Failed to fetch game story for game id {}: {}",
                            game_id,
                            err
                        );
                    }
                }
            }
        });

        futures::future::join_all(fetches).await;
    }
}

#[async_trait::async_trait]
impl Source for GameStorySource {
    async fn run(mut self: Box<Self>, tx: Sender<AppEvent>, cancel: CancellationToken) {
        let mut interval = tokio::time::interval(self.fetch_interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,

                Some(cmd) = self.rx.recv() => {
                    match cmd {
                        GameStoryCommand::SetGameIds(mut ids) => {
                            ids.sort();
                            log::info!("Received game ids: {:?}", ids);
                            let mut current = self.game_ids.clone();
                            current.sort();
                            // Only fetch if game ids changed since this command is called on every GamesUpdate event
                            if ids != current {
                                log::info!("Fetching game story because game ids changed");
                                self.game_ids = ids;
                                self.fetch(&tx).await;
                                interval.reset();
                            }
                        },
                        GameStoryCommand::SetInterval(new_interval) => {
                            if new_interval != self.fetch_interval {
                                log::info!("Setting game story interval to {:?}", new_interval);
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
