use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

use super::{AppEvent, Source};

pub enum BoxscoreCommand {
    SetGameId(u32),
}

pub struct BoxscoreSource {
    rx: Receiver<BoxscoreCommand>,
    game_id: Option<u32>,
}
impl BoxscoreSource {
    pub fn new(rx: Receiver<BoxscoreCommand>) -> Self {
        Self {
            rx,
            game_id: None,
        }
    }

    async fn fetch(&self, tx: &Sender<AppEvent>) {
        if let Some(id) = self.game_id {
            let url = format!("https://api-web.nhle.com/v1/gamecenter/{}/boxscore", id);

            match reqwest::get(&url).await {
                Ok(resp) => {
                    if let Ok(body) = resp.text().await {
                        let _ = tx.send(AppEvent::BoxscoreUpdate(body)).await;
                    }
                }
                Err(err) => {
                    log::info!("Failed to fetch boxscore: {}", err);
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl Source for BoxscoreSource {
    async fn run(mut self: Box<Self>, tx: Sender<AppEvent>, cancel: CancellationToken) {
        let mut interval = tokio::time::interval(Duration::from_secs(10));

        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,

                Some(cmd) = self.rx.recv() => {
                    match cmd {
                        BoxscoreCommand::SetGameId(id) => {
                            self.game_id = Some(id);
                            self.fetch(&tx).await;
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
