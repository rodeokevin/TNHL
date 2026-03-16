use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Row, Table, TableState},
};

use serde::Deserialize;

use crate::app::{App, PaneFocus};

#[derive(Debug, Deserialize)]
struct StandingsResponse {
    standings: Vec<TeamRecord>,
}

#[derive(Debug, Deserialize)]
pub struct TeamRecord {
    #[serde(rename = "teamName")]
    pub team_name: TeamName,
    #[serde(rename = "teamAbbrev")]
    pub team_abbr: TeamAbbrev,
    #[serde(rename = "gamesPlayed")]
    pub games_played: u32,
    pub wins: u32,
    pub losses: u32,
    #[serde(rename = "otLosses")]
    pub ot_losses: u32,
    pub points: u32,
    #[serde(rename = "pointPctg")]
    pub point_pctg: f64,
    #[serde(rename = "regulationWins")]
    pub regulation_wins: u32,
    #[serde(rename = "regulationPlusOtWins")]
    pub regulation_plus_ot_wins: u32,
    #[serde(rename = "goalFor")]
    pub goal_for: u32,
    #[serde(rename = "goalAgainst")]
    pub goal_against: u32,
    #[serde(rename = "homeWins")]
    pub home_wins: u32,
    #[serde(rename = "homeOtLosses")]
    pub home_ot_losses: u32,
    #[serde(rename = "homeLosses")]
    pub home_losses: u32,
    #[serde(rename = "roadWins")]
    pub road_wins: u32,
    #[serde(rename = "roadOtLosses")]
    pub road_ot_losses: u32,
    #[serde(rename = "roadLosses")]
    pub road_losses: u32,
    #[serde(rename = "shootoutWins")]
    pub shootout_wins: u32,
    #[serde(rename = "shootoutLosses")]
    pub shootout_losses: u32,
    #[serde(rename = "l10Wins")]
    pub l10_wins: u32,
    #[serde(rename = "l10OtLosses")]
    pub l10_ot_losses: u32,
    #[serde(rename = "l10Losses")]
    pub l10_losses: u32,
    #[serde(rename = "streakCode")]
    pub streak_code: String,
    #[serde(rename = "streakCount")]
    pub streak_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct TeamName {
    pub default: String,
    // fr field is optional
}

#[derive(Debug, Deserialize)]
pub struct TeamAbbrev {
    pub default: String,
}

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // main display area
            Constraint::Length(3), // footer
        ])
        .split(frame.area());

    // Split main area into menu + main content
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // sidebar
            Constraint::Percentage(75), // main content
        ])
        .split(chunks[0]);
    render_menu(frame, app, main_chunks[0]);
    render_league_standings(frame, app, main_chunks[1]);

    // Render footer
    let footer_block = Block::bordered();
    let commands = Line::from("Quit [q]");
    let footer = Paragraph::new(commands).block(footer_block);

    frame.render_widget(footer, chunks[1]);
}

fn render_menu(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == PaneFocus::Menu;
    let border_style = if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let menu_items = vec![
        ListItem::new("Standings"),
        ListItem::new("Teams"),
        ListItem::new("Games"),
    ];

    let list = List::new(menu_items)
        .block(Block::bordered().title(" Menu ").border_style(border_style))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    let mut state = ListState::default();
    state.select(Some(app.menu_index));

    frame.render_stateful_widget(list, area, &mut state);
}

fn render_league_standings(frame: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == PaneFocus::Information;
    let border_style = if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    if let Some(json) = &app.league_standings {
        let header = Row::new([
            "#", "Team", "GP", "W", "L", "OT", "PTS", "P%", "RW", "ROW", "GF", "GA", "DIFF",
            "HOME", "AWAY", "S/O", "L10", "STRK",
        ])
        .style(Style::new().bold())
        .bottom_margin(1);

        let rows = map_standings_to_rows(json);
        let widths = [
            Constraint::Length(4),
            Constraint::Length(5),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(5),
            Constraint::Length(6),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(5),
            Constraint::Length(8),
            Constraint::Length(5),
        ];

        let table = Table::new(rows, widths)
            .block(Block::bordered().title(" League Standings ").border_style(border_style))
            .header(header)
            .column_spacing(1)
            .row_highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(table, area, &mut app.league_standings_table_state);
    }
}

// Helper function to map the standings JSON into table rows
pub fn map_standings_to_rows(json: &str) -> Vec<Row<'static>> {
    let response: StandingsResponse = match serde_json::from_str(json) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Failed to parse standings JSON: {}", e);
            return Vec::new();
        }
    };

    response
        .standings
        .iter()
        .enumerate()
        .map(|(i, team)| {
            Row::new(vec![
                (i + 1).to_string(),
                team.team_abbr.default.clone(),
                team.games_played.to_string(),
                team.wins.to_string(),
                team.losses.to_string(),
                team.ot_losses.to_string(),
                team.points.to_string(),
                team.point_pctg.to_string(),
                team.regulation_wins.to_string(),
                team.regulation_plus_ot_wins.to_string(),
                team.goal_for.to_string(),
                team.goal_against.to_string(),
                ((team.goal_for as i32) - (team.goal_against as i32)).to_string(),
                format!(
                    "{}-{}-{}",
                    team.home_wins, team.home_losses, team.home_ot_losses
                ),
                format!(
                    "{}-{}-{}",
                    team.road_wins, team.road_losses, team.road_ot_losses
                ),
                format!("{}-{}", team.shootout_wins, team.shootout_losses),
                format!(
                    "{}-{}-{}",
                    team.l10_wins, team.l10_losses, team.l10_ot_losses
                ),
                format!("{}{}", team.streak_code, team.streak_count),
            ])
        })
        .collect()
}
