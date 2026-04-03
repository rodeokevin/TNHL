use crate::models::boxscore::BoxscoreResponse;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct BoxscoreState {
    pub games: HashMap<u32, GameBoxscore>,
}

#[derive(Debug, Clone)]
pub struct GameBoxscore {
    pub game_id: u32,
    pub data: BoxscoreResponse,
    pub derived_stats: Option<DerivedStats>,
}

#[derive(Debug, Clone, Default)]
pub struct DerivedStats {
    pub away: TeamStats,
    pub home: TeamStats,
}

#[derive(Debug, Clone, Default)]
pub struct TeamStats {
    pub hits: u16,
    pub shots_on_goal: u16,
    pub penalty_minutes: u16,
    pub blocked_shots: u16,
    pub giveaways: u16,
    pub takeaways: u16,
}

impl GameBoxscore {
    pub fn compute_totals(data: &BoxscoreResponse) -> Option<DerivedStats> {
        let player_stats = data.player_by_game_stats.as_ref()?;
        let mut result = DerivedStats::default();
        for (team, stats) in [
            (&player_stats.away_team, &mut result.away),
            (&player_stats.home_team, &mut result.home),
        ] {
            for p in team.forwards.iter().chain(&team.defense) {
                stats.hits += p.hits as u16;
                stats.shots_on_goal += p.sog as u16;
                stats.penalty_minutes += p.pim as u16;
                stats.blocked_shots += p.blocked_shots as u16;
                stats.giveaways += p.giveaways as u16;
                stats.takeaways += p.takeaways as u16;
            }
            for g in &team.goalies {
                stats.penalty_minutes += g.pim as u16;
            }
        }
        Some(result)
    }
}
