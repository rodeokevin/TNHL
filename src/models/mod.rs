use serde::Deserialize;

pub mod games;
pub mod playoff_bracket;
pub mod plays;
pub mod standings;
pub mod team_stats;

#[derive(Deserialize, Debug)]
pub struct TeamName {
    pub default: String,
}
