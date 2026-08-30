use crate::models::{
    TeamName,
    games::games::{PeriodDescriptor, PlayerName},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaysResponse {
    pub game_type: u8,
    pub away_team: PlayByPlayTeam,
    pub home_team: PlayByPlayTeam,
    pub plays: Vec<PlayData>,
    pub roster_spots: Vec<RosterPlayer>,
}

impl PlaysResponse {
    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayData {
    pub event_id: u32,
    pub period_descriptor: Option<PeriodDescriptor>,
    pub time_in_period: String,
    pub time_remaining: String,
    pub type_desc_key: TypeDescKey,
    pub details: Option<PlayDetails>,
}

/// Per-play details. Fields are populated depending on the play type; player
/// references are IDs resolved against `PlaysResponse::roster_spots`.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayDetails {
    pub event_owner_team_id: Option<u32>,
    // Faceoff
    pub winning_player_id: Option<u32>,
    pub losing_player_id: Option<u32>,
    // Hit
    pub hitting_player_id: Option<u32>,
    pub hittee_player_id: Option<u32>,
    // Shots
    pub shooting_player_id: Option<u32>,
    pub blocking_player_id: Option<u32>,
    pub goalie_in_net_id: Option<u32>,
    pub shot_type: Option<String>,
    // Goal
    pub scoring_player_id: Option<u32>,
    pub scoring_player_total: Option<u32>,
    pub assist1_player_id: Option<u32>,
    pub assist2_player_id: Option<u32>,
    pub away_score: Option<u32>,
    pub home_score: Option<u32>,
    // Penalty
    pub committed_by_player_id: Option<u32>,
    pub drawn_by_player_id: Option<u32>,
    pub served_by_player_id: Option<u32>,
    pub desc_key: Option<DescKey>,
    pub duration: Option<u32>,
    // Giveaway / takeaway
    pub player_id: Option<u32>,
    // Stoppage
    pub reason: Option<String>,
    pub secondary_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TypeDescKey {
    Hit,
    Stoppage,
    Faceoff,
    BlockedShot,
    Misconduct,
    MissedShot,
    ShotOnGoal,
    DelayedPenalty,
    Penalty,
    Giveaway,
    Takeaway,
    Goal,
    PeriodStart,
    PeriodEnd,
    ShootoutComplete,
    GameEnd,
    #[serde(other)]
    Unknown,
}

/// Description key for penalties.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DescKey {
    // Minors
    Boarding,
    Charging,
    Clipping,
    CrossChecking,
    Diving,
    Elbowing,
    Embellishment,
    Fighting,
    HeadButting,
    Holding,
    HoldingTheStick,
    Hooking,
    Interference,
    InterferenceGoalkeeper,
    InterferenceBench,
    Kneeing,
    Roughing,
    RoughingRemovingOpponentsHelmet,
    Slashing,
    Tripping,
    HighSticking,
    Instigator,
    InstigatorMisconduct,
    InstigatorFaceShield,
    UnsportsmanlikeConduct,
    Spearing,
    CheckingFromBehind,
    IllegalCheckToHead,
    ClosingHandOnPuck,
    BrokenStick,
    IllegalEquipment,
    // Double minors
    HighStickingDoubleMinor,
    RoughingDoubleMinor,
    SpearingDoubleMinor,
    ButtEndingDoubleMinor,
    // Delay of game
    DelayingGame,
    DelayingGameBench,
    DelayingGamePuckOverGlass,
    DelayingGameUnsuccessfulChallenge,
    DelayingGameSmotheringPuck,
    DelayingGameFaceOffViolation,
    DelayingGameBenchFaceOffViolation,
    DelayingGameIllegalPlayByGoalie,
    DelayingGameEquipment,
    // Bench / team
    Bench,
    TooManyMenOnTheIce,
    UnsportsmanlikeConductBench,
    IllegalStickBench,
    IllegalSubstitution,
    InterferenceWithOfficial,
    // Majors / misconduct
    AttemptToInjure,
    Misconduct,
    GameMisconduct,
    GrossMisconduct,
    GameMisconductHeadCoach,
    MatchPenalty,
    MatchPenatly10Minutes,
    // Goalie / equipment
    GoalieLeaveCrease,
    GoalieRemovedOwnMask,
    GoalieParticipationBeyondCenter,
    PuckThrownForwardGoalkeeper,
    ThrowingEquipment,
    IllegalStick,
    PlayingWithoutAHelmet,
    // Penalty shot
    PsHookingOnBreakaway,
    PsSlashOnBreakaway,
    PsHoldingOnBreakaway,
    PsTrippingOnBreakaway,
    PsThrowingObjectAtPuck,
    PsCoveringPuckInCrease,
    PsGoalkeeperDisplacedNet,
    PsNetDisplaced,
    PenaltyShot,
    PenaltyShotMinor,
    // Other
    AbuseOfOfficials,
    AbusiveLanguage,
    Aggressor,
    IneligiblePlayer,
    Minor,
    UnsportsmanlikeConductCoach,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RosterPlayer {
    pub team_id: u32,
    pub player_id: u32,
    pub first_name: PlayerName,
    pub last_name: PlayerName,
    pub sweater_number: u8,
    pub position_code: String,
}

impl RosterPlayer {
    /// Display initial of first name, last name with sweater number
    pub fn short_name(&self) -> String {
        let initial = self.first_name.default.chars().next().unwrap_or(' ');
        format!(
            "{}. {} (#{})",
            initial, self.last_name.default, self.sweater_number
        )
    }

    /// Display last name with sweater number
    pub fn last_name(&self) -> String {
        format!(
            "{} (#{})",
            self.last_name.default, self.sweater_number
        )
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayByPlayTeam {
    pub id: u32,
    pub abbrev: String,
    pub common_name: TeamName,
}
