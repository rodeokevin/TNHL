use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

use crate::sources::BracketResponse;
use crate::{AppEvent, Source};

pub enum BracketCommand {
    SetYear(i32),
}

pub struct BracketSource {
    rx: Receiver<BracketCommand>,
    current_year: i32,
}
impl BracketSource {
    pub fn new(rx: Receiver<BracketCommand>, current_year: i32) -> Self {
        Self { rx, current_year }
    }

    async fn fetch(&self, tx: &Sender<AppEvent>) {
        let url = format!(
            "https://api-web.nhle.com/v1/playoff-bracket/{}",
            self.current_year.to_string()
        );

        match reqwest::get(&url).await {
            Ok(resp) => {
                if let Ok(body) = resp.text().await {
                    // Parse the JSON
                    match BracketResponse::from_json(&body) {
                        Ok(parsed_playoff_bracket) => {
                            log::info!("Bracket data successfully parsed!");
                            let _ = tx
                                .send(AppEvent::BracketUpdate(parsed_playoff_bracket))
                                .await;
                            log::info!("Sent Bracket data to app");
                        }
                        Err(e) => log::error!("Failed to parse Bracket: {}", e),
                    }
                }
            }
            Err(err) => log::info!("Failed to fetch Bracket: {}", err),
        }
    }
}

#[async_trait::async_trait]
impl Source for BracketSource {
    async fn run(mut self: Box<Self>, tx: Sender<AppEvent>, cancel: CancellationToken) {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,
                Some(cmd) = self.rx.recv() => {
                    match cmd {
                        BracketCommand::SetYear(year) => {
                            self.current_year = year;
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
