use crate::app::App;
use crate::models::team_stats::{Goalie, Skater};
use crate::state::app_state::PaneFocus;
use crate::ui::render::BORDER_FOCUSED_COLOR;

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Row, Table},
};

const SKATERS_COLUMNS: [&str; 17] = [
    "Player", "POS", "GP", "G", "A", "P", "+/-", "PIM", "PPG", "SHG", "GWG", "OTG", "S", "S%",
    "TOI/G", "SHIFT/G", "FO%",
];

const GOALIES_COLUMNS: [&str; 18] = [
    "Player", "GP", "GS", "W", "L", "T", "OTL", "GAA", "SV%", "SA", "SV", "GA", "SO", "G", "A",
    "P", "PIM", "TOI",
];

const SKATERS_COLUMNS_WIDTHS: [Constraint; 17] = [
    Constraint::Min(20),   // Name
    Constraint::Length(4), // POS
    Constraint::Length(3), // GP
    Constraint::Length(4), // G
    Constraint::Length(4), // A
    Constraint::Length(4), // P
    Constraint::Length(4), // +/-
    Constraint::Length(4), // PIM
    Constraint::Length(4), // PPG
    Constraint::Length(4), // SHG
    Constraint::Length(4), // GWG
    Constraint::Length(4), // OTG
    Constraint::Length(4), // S
    Constraint::Length(5), // S%
    Constraint::Length(6), // TOI/G
    Constraint::Length(7), // SHIFT/G
    Constraint::Length(5), // FO%
];

const BOXSCORE_GOALIES_COLUMN_WIDTHS: [Constraint; 18] = [
    Constraint::Min(20),   // Name
    Constraint::Length(4), // GP
    Constraint::Length(4), // GS
    Constraint::Length(4), // W
    Constraint::Length(4), // L
    Constraint::Length(3), // T
    Constraint::Length(3), // OTL
    Constraint::Length(7), // GAA
    Constraint::Length(7), // SV%
    Constraint::Length(6), // SA
    Constraint::Length(6), // SV
    Constraint::Length(5), // GA
    Constraint::Length(4), // SO
    Constraint::Length(3), // G
    Constraint::Length(3), // A
    Constraint::Length(3), // P
    Constraint::Length(4), // PIM
    Constraint::Length(8), // TOI
];

pub fn render_team_stats(frame: &mut Frame, app: &mut App, area: Rect) {
    // Pass visible rows to team stats state
    app.state.team_stats.visible_rows = area.height.saturating_sub(3) as usize;
    let focused = app.state.focus == PaneFocus::Content;
    let border_style = if focused {
        Style::default()
            .fg(BORDER_FOCUSED_COLOR)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let show_skaters = app.state.team_stats.show_skaters;

    if let Some(data) = &app.state.team_stats.team_stats_data {
        let (rows, widths, header): (Vec<Row<'static>>, &[Constraint], &[&str]) = if show_skaters {
            let rows = map_skater_rows(&data.skaters);
            (rows, &SKATERS_COLUMNS_WIDTHS, &SKATERS_COLUMNS)
        } else {
            let rows = map_goalie_rows(&data.goalies);
            (rows, &BOXSCORE_GOALIES_COLUMN_WIDTHS, &GOALIES_COLUMNS)
        };

        let title = format!(
            " {} {} ",
            app.state.team_stats.team_picker.current_team,
            if show_skaters { "Skaters" } else { "Goalies" }
        );

        let table = Table::new(rows, widths)
            .block(Block::bordered().title(title).border_style(border_style))
            .header(
                Row::new(header.iter().map(|s| s.to_string()).collect::<Vec<_>>())
                    .style(Style::new().bold().add_modifier(Modifier::UNDERLINED)),
            )
            .column_spacing(1)
            .row_highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(table, area, &mut app.state.team_stats.table_state);
    }
}

fn map_skater_rows(players: &[Skater]) -> Vec<Row<'static>> {
    players
        .iter()
        .map(|p| {
            let name = p.first_name.default.clone() + " " + &p.last_name.default.clone();
            Row::new(vec![
                name,
                p.position_code.to_string(),
                p.games_played.to_string(),
                p.goals.to_string(),
                p.assists.to_string(),
                p.points.to_string(),
                p.plus_minus.to_string(),
                p.penalty_minutes.to_string(),
                p.power_play_goals.to_string(),
                p.shorthanded_goals.to_string(),
                p.game_winning_goals.to_string(),
                p.overtime_goals.to_string(),
                p.shots.to_string(),
                format!("{:.1}", p.shooting_pctg * 100.0),
                format!("{:.1}", p.avg_time_on_ice_per_game / 60.0), // the time is in seconds
                format!("{:.1}", p.avg_shifts_per_game),
                if p.faceoff_win_pctg == 0.0 {
                    "0".to_string()
                } else {
                    format!("{:.1}", p.faceoff_win_pctg * 100.0)
                },
            ])
        })
        .collect()
}

fn map_goalie_rows(players: &[Goalie]) -> Vec<Row<'static>> {
    players
        .iter()
        .map(|p| {
            let name = p.first_name.default.clone() + " " + &p.last_name.default.clone();
            let minutes = p.time_on_ice / 60;
            let seconds = p.time_on_ice % 60;
            let toi = format!("{}:{:02}", minutes, seconds);
            Row::new(vec![
                name,
                p.games_played.to_string(),
                p.games_started.to_string(),
                p.wins.to_string(),
                p.losses.to_string(),
                p.ties
                    .map(|ties| ties.to_string())
                    .unwrap_or_else(|| "--".to_string()),
                p.overtime_losses
                    .map(|losses| losses.to_string())
                    .unwrap_or_else(|| "--".to_string()),
                format!("{:.2}", p.goals_against_average),
                format!("{:.3}", p.save_percentage),
                p.shots_against.to_string(),
                p.saves.to_string(),
                p.goals_against.to_string(),
                p.shutouts.to_string(),
                p.goals.to_string(),
                p.assists.to_string(),
                p.points.to_string(),
                p.penalty_minutes.to_string(),
                toi,
            ])
        })
        .collect()
}
