use serde::Deserialize;
use std::str::FromStr;
use strum_macros::{Display, EnumString};

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

#[derive(Debug)]
pub enum InputError {
    InvalidLength,
    InvalidTeam,
}

pub struct TeamPickerState {
    pub is_valid: bool,
    /// Current user input in the date picker
    pub text: String,
    /// Current team
    pub current_team: TeamAbbrev,
}

impl TeamPickerState {
    /// Validate the input date
    pub fn validate_input(&mut self) -> Result<TeamAbbrev, InputError> {
        let input: String = self.text.drain(..).collect();
        let input = input.trim();

        if input.len() != 3 || !input.chars().all(|c| c.is_ascii_alphabetic()) {
            self.is_valid = false;
            return Err(InputError::InvalidLength);
        }

        TeamAbbrev::from_str(input).map_err(|_| {
            self.is_valid = false;
            InputError::InvalidTeam
        })
    }
    /// Set the team from the validated input string from the team picker
    pub fn set_team_from_valid_input(&mut self, team: TeamAbbrev) {
        self.current_team = team;
    }
}

impl Default for TeamPickerState {
    fn default() -> Self {
        Self {
            is_valid: true,
            text: String::new(),
            current_team: TeamAbbrev::default(),
        }
    }
}
