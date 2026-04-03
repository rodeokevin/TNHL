use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

use super::{AppEvent, Source};

pub enum GamesCommand {
    SetDate(String),
}

pub struct GamesSource {
    rx: Receiver<GamesCommand>,
    current_date: String,
}
impl GamesSource {
    pub fn new(rx: Receiver<GamesCommand>) -> Self {
        Self {
            rx,
            current_date: "now".to_string(),
        }
    }

    async fn fetch(&self, tx: &Sender<AppEvent>) {
        let url = format!("https://api-web.nhle.com/v1/score/{}", self.current_date);

        match reqwest::get(&url).await {
            Ok(resp) => {
                if let Ok(body) = resp.text().await {
                    let _ = tx.send(AppEvent::GamesUpdate(body)).await;
                }
            }
            Err(err) => {
                log::info!("Failed to fetch games: {}", err);
            }
        }
    }
}

#[async_trait::async_trait]
impl Source for GamesSource {
    async fn run(mut self: Box<Self>, tx: Sender<AppEvent>, cancel: CancellationToken) {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
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
