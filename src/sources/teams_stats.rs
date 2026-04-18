use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

use super::{AppEvent, Source};
use crate::models::team_stats::TeamStatsResponse;
use crate::sources::FetchInterval;
use crate::state::team_stats::team_picker::TeamAbbrev;

pub enum TeamStatsCommand {
    SetTeam(TeamAbbrev),
    SetYear(i32),
    SetInterval(Duration),
}

pub struct TeamStatsSource {
    rx: Receiver<TeamStatsCommand>,
    current_team: TeamAbbrev,
    current_year: i32,
    fetch_interval: Duration,
}
impl TeamStatsSource {
    pub fn new(rx: Receiver<TeamStatsCommand>, current_team: TeamAbbrev, current_year: i32) -> Self {
        Self {
            rx,
            current_team,
            current_year,
            fetch_interval: FetchInterval::InfoShortInterval.as_duration(),
        }
    }

    async fn fetch(&self, tx: &Sender<AppEvent>) {
        let url = format!(
            "https://api-web.nhle.com/v1/club-stats/{}/{}{}/2",
            self.current_team.to_string(),
            self.current_year - 1,
            self.current_year,
        );

        match reqwest::get(&url).await {
            Ok(resp) => {
                if let Ok(body) = resp.text().await {
                    // Parse the JSON
                    match TeamStatsResponse::from_json(&body) {
                        Ok(mut parsed_team_stats) => {
                            log::info!("Team stats data successfully parsed!");
                            // Sort by points for skaters
                            parsed_team_stats
                                .skaters
                                .sort_by_key(|s| std::cmp::Reverse(s.points));
                            // Sort by games played for goalies
                            parsed_team_stats
                                .goalies
                                .sort_by_key(|s| std::cmp::Reverse(s.games_played));
                            let _ = tx.send(AppEvent::TeamStatsUpdate(parsed_team_stats)).await;
                            log::info!("Sent team stats data to app");
                        }
                        Err(e) => log::error!("Failed to parse team stats: {}", e),
                    }
                }
            }
            Err(err) => log::info!("Failed to fetch team stats: {}", err),
        }
    }
}

#[async_trait::async_trait]
impl Source for TeamStatsSource {
    async fn run(mut self: Box<Self>, tx: Sender<AppEvent>, cancel: CancellationToken) {
        let mut interval = tokio::time::interval(self.fetch_interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,
                Some(cmd) = self.rx.recv() => {
                    match cmd {
                        TeamStatsCommand::SetTeam(team) => {
                            self.current_team = team;
                            self.fetch(&tx).await;
                            interval.reset();
                        }
                        TeamStatsCommand::SetYear(year) => {
                            self.current_year = year;
                            self.fetch(&tx).await;
                            interval.reset();
                        }
                        TeamStatsCommand::SetInterval(new_interval) => {
                            if new_interval != self.fetch_interval {
                                log::info!("Setting team stats interval to {:?}", new_interval);
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
