use serde::Deserialize;
use strum_macros::{Display, EnumString};

pub mod games;
pub mod playoffs;
pub mod plays;
pub mod standings;
pub mod team_stats;

#[derive(Deserialize, Debug)]
pub struct TeamName {
    pub default: String,
}

#[derive(Deserialize, Debug)]
pub struct PlaceName {
    pub default: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Display, EnumString, Deserialize)]
#[strum(ascii_case_insensitive)]
pub enum TeamAbbrev {
    AFM, // Atlanta Flames (defunct)
    ANA, // Anaheim Ducks
    ARI, // Arizona Coyotes (defunct)
    ATL, // Atlanta Thrasher (defunct)
    BOS,
    BRK, // Brooklyn Americans (defunct)
    BUF,
    CAR,
    CBJ,
    CGS, // Bay Area Seals/California Golden Seals (Defunct)
    CGY,
    CHI,
    CLE, // Cleveland Barons (defunct)
    CLR, // Colorado Rockies (defunct)
    COL,
    DAL,
    DCG, // Detroit Cougars (defunct)
    DET,
    DFL, // Detroit Falcons (defunct)
    EDM,
    FLA,
    HAM, // Hamilton Tigers (defunct)
    HFD,
    KCS, // Kansas City Scouts (defunct)
    LAK,
    MIN,
    MMR, // Montreal Marrons (defunct)
    MNS, // Minnesota North Stars (defunct)
    #[default]
    MTL,
    MWN, // Montreal Wanderers (defunct)
    NJD,
    NSH,
    NYA, // New York Americans (defunct)
    NYI,
    NYR,
    OAK, // California/Oakland Seals (defunct)
    OTT,
    PHI,
    PHX, // Phoenix Coyotes (defunct)
    PIR, // Pittsburgh Pirates (defunct)
    PIT,
    QBD, // Quebec Bulldogs (defunct)
    QUA, // Philadelphia Quakers(defunct)
    QUE, // Quebec Nordiques (defunct)
    SEA,
    SEN, // Ottawa Senators (original)
    SLE, // St. Louis Eagles (defunct)
    SJS,
    SMT,
    STL,
    TAN, // Toronto Hockey Club/Toronto Arenas (defunct)
    TBD, // To be determined (used in playoff series)
    TBL,
    TOR,
    TSP, // Toronto St. Patricks (defunct)
    UTA,
    VAN,
    VGK,
    WIN, // Winnipeg Jets (original)
    WPG,
    WSH,
}
