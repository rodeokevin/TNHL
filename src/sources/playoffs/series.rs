use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

use crate::sources::SeriesResponse;
use crate::{AppEvent, Source};

pub enum SeriesCommand {
    SetYear(i32),
    SetSeries(Option<char>),
}

pub struct SeriesSource {
    rx: Receiver<SeriesCommand>,
    current_year: i32,
    series_letter: Option<char>,
}
impl SeriesSource {
    pub fn new(
        rx: Receiver<SeriesCommand>,
        current_year: i32,
        series_letter: Option<char>,
    ) -> Self {
        Self {
            rx,
            current_year,
            series_letter,
        }
    }

    async fn fetch(&self, tx: &Sender<AppEvent>) {
        if let Some(letter) = &self.series_letter {
            let url = format!(
                "https://api-web.nhle.com/v1/schedule/playoff-series/{}{}/{}",
                (self.current_year - 1).to_string(),
                self.current_year.to_string(),
                letter,
            );
            match reqwest::get(&url).await {
                Ok(resp) => {
                    if let Ok(body) = resp.text().await {
                        // Parse the JSON
                        match SeriesResponse::from_json(&body) {
                            Ok(parsed_series) => {
                                log::info!("Series data successfully parsed!");
                                let _ = tx.send(AppEvent::SeriesUpdate(parsed_series)).await;
                                log::info!("Sent series data to app");
                            }
                            Err(e) => log::error!("Failed to parse series: {}", e),
                        }
                    }
                }
                Err(err) => log::info!("Failed to fetch series: {}", err),
            }
        } else {
            log::info!("Not fetching series data because no series is selected");
        }
    }
}

#[async_trait::async_trait]
impl Source for SeriesSource {
    async fn run(mut self: Box<Self>, tx: Sender<AppEvent>, cancel: CancellationToken) {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,
                Some(cmd) = self.rx.recv() => {
                    match cmd {
                        SeriesCommand::SetYear(year) => {
                            self.current_year = year;
                            // No series should be selected when the year changes
                            self.series_letter = None;
                        }
                        SeriesCommand::SetSeries(letter) => {
                            if let Some(letter) = letter {
                                log::info!("Fetching new series data for {} series: {}", self.current_year, letter);
                                self.series_letter = Some(letter);
                                self.fetch(&tx).await;
                                interval.reset();
                            } else {
                                log::info!("Series letter not set because it was None");
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
