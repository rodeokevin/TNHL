use tokio::sync::mpsc::Sender;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

use super::{AppEvent, Source};

pub struct StandingsSource;
impl StandingsSource {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Source for StandingsSource {
    async fn run(self: Box<Self>, tx: Sender<AppEvent>, cancel: CancellationToken) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,
                _ = interval.tick() => {
                    match reqwest::get("https://api-web.nhle.com/v1/standings/now").await {
                        Ok(resp) => {
                            if let Ok(body) = resp.text().await {
                                let _ = tx.send(AppEvent::StandingsUpdate(body)).await;
                            }
                        }
                        Err(err) => {
                            log::info!("Failed to fetch standings: {}", err);
                        }
                    }
                }
            }
        }
    }
}
