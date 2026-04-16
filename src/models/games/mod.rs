use serde::Deserialize;
use std::fmt;

pub mod boxscore;
pub mod game_story;
pub mod games;

#[derive(Debug, Clone, Deserialize)]
pub enum Position {
    #[serde(rename = "L")]
    LeftWing,
    #[serde(rename = "R")]
    RightWing,
    #[serde(rename = "C")]
    Center,
    #[serde(rename = "D")]
    Defense,
    #[serde(rename = "G")]
    Goalie,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Position::LeftWing => "LW",
            Position::RightWing => "RW",
            Position::Center => "C",
            Position::Defense => "D",
            Position::Goalie => "G",
        };
        write!(f, "{}", s)
    }
}
