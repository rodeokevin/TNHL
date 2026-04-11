use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

use super::{AppEvent, Source};
use crate::sources::StandingsResponse;

pub enum StandingsCommand {
    SetDate(String),
}

pub struct StandingsSource {
    rx: Receiver<StandingsCommand>,
    current_date: String,
}
impl StandingsSource {
    pub fn new(rx: Receiver<StandingsCommand>, current_date: String) -> Self {
        Self { rx, current_date }
    }

    async fn fetch(&self, tx: &Sender<AppEvent>) {
        let url = format!(
            "https://api-web.nhle.com/v1/standings/{}",
            self.current_date
        );

        match reqwest::get(&url).await {
            Ok(resp) => {
                if let Ok(body) = resp.text().await {
                    // Parse the JSON
                    match StandingsResponse::from_json(&body) {
                        Ok(parsed_standings) => {
                            log::info!("Standings data successfully parsed!");
                            let _ = tx.send(AppEvent::StandingsUpdate(parsed_standings)).await;
                            log::info!("Sent standings data to app");
                        }
                        Err(e) => log::error!("Failed to parse standings: {}", e),
                    }
                }
            }
            Err(err) => {
                log::info!("Failed to fetch standings: {}", err);
            }
        }
    }
}

#[async_trait::async_trait]
impl Source for StandingsSource {
    async fn run(mut self: Box<Self>, tx: Sender<AppEvent>, cancel: CancellationToken) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,
                Some(cmd) = self.rx.recv() => {
                    match cmd {
                        StandingsCommand::SetDate(date) => {
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
