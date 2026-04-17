use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

use super::{AppEvent, Source};
use crate::sources::PlayoffBracketResponse;

pub enum PlayoffBracketCommand {
    SetYear(u16),
}

pub struct PlayoffBracketSource {
    rx: Receiver<PlayoffBracketCommand>,
    current_year: u16,
}
impl PlayoffBracketSource {
    pub fn new(rx: Receiver<PlayoffBracketCommand>, current_year: u16) -> Self {
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
                    match PlayoffBracketResponse::from_json(&body) {
                        Ok(parsed_playoff_bracket) => {
                            log::info!("PlayoffBracket data successfully parsed!");
                            let _ = tx
                                .send(AppEvent::PlayoffBracketUpdate(parsed_playoff_bracket))
                                .await;
                            log::info!("Sent PlayoffBracket data to app");
                        }
                        Err(e) => log::error!("Failed to parse PlayoffBracket: {}", e),
                    }
                }
            }
            Err(err) => log::info!("Failed to fetch PlayoffBracket: {}", err),
        }
    }
}

#[async_trait::async_trait]
impl Source for PlayoffBracketSource {
    async fn run(mut self: Box<Self>, tx: Sender<AppEvent>, cancel: CancellationToken) {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,
                Some(cmd) = self.rx.recv() => {
                    match cmd {
                        PlayoffBracketCommand::SetYear(year) => {
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
