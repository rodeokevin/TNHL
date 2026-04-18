use serde::Deserialize;
use strum_macros::{Display, EnumString};

pub mod games;
pub mod playoff_bracket;
pub mod plays;
pub mod standings;
pub mod team_stats;

#[derive(Deserialize, Debug)]
pub struct TeamName {
    pub default: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Display, EnumString, Deserialize)]
#[strum(ascii_case_insensitive)]
pub enum TeamAbbrev {
    ANA,
    ARI,
    BOS,
    BUF,
    CAR,
    CBJ,
    CGY,
    CHI,
    COL,
    DAL,
    DET,
    EDM,
    FLA,
    HFD,
    LAK,
    MIN,
    MNS,
    #[default]
    MTL,
    NJD,
    NSH,
    NYI,
    NYR,
    OAK,
    OTT,
    PHI,
    PHX,
    PIT,
    SEA,
    SEN,
    SJS,
    SMT,
    STL,
    TBL,
    TOR,
    UTA,
    VAN,
    VGK,
    WIN,
    WPG,
    WSH,
}
