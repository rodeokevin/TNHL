use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, List, ListItem, ListState, Paragraph, Row, Table, Tabs},
};

use serde::Deserialize;

use crate::app::{App, ConferenceType, PaneFocus, StandingsFocus};

#[derive(Debug, Deserialize)]
struct StandingsResponse {
    standings: Vec<TeamRecord>,
}

#[derive(Debug, Deserialize)]
pub struct TeamRecord {
    #[serde(rename = "teamName")]
    pub team_name: TeamName,
    #[serde(rename = "teamAbbrev")]
    pub team_abbrev: TeamAbbrev,
    #[serde(rename = "conferenceAbbrev")]
    pub conference_abbrev: String,
    #[serde(rename = "conferenceSequence")]
    pub conference_sequence: u32,
    #[serde(rename = "wildcardSequence")]
    pub wildcard_sequence: u32,
    #[serde(rename = "divisionSequence")]
    pub division_sequence: u32,
    #[serde(rename = "leagueSequence")]
    pub league_sequence: u32,
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
    let content_menu_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(15), // sidebar
            Constraint::Percentage(85), // content
        ])
        .split(chunks[0]);
    render_menu(frame, app, content_menu_chunks[0]);
    
    // Split content chunk into tab + content
    let tab_content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // tabs
            Constraint::Min(1),    // table
        ])
        .split(content_menu_chunks[1]);

    let titles = [" Wild Card ", " Division ", " Conference ", " League "]
        .iter()
        .map(|t| Line::from(*t))
        .collect::<Vec<_>>();

    let selected_standings = match app.standings_type {
        StandingsFocus::WildCard => 0,
        StandingsFocus::Division => 1,
        StandingsFocus::Conference => 2,
        StandingsFocus::League => 3,
    };

    let tabs = Tabs::new(titles)
        .select(selected_standings)
        .block(Block::bordered())
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, tab_content_chunks[0]);

    if app.menu_index == 0 {
        match app.standings_type {
            StandingsFocus::WildCard => render_wildcard_standings(frame, app, tab_content_chunks[1]),
            StandingsFocus::Division => render_division_standings(frame, app, tab_content_chunks[1]),
            StandingsFocus::Conference => render_conference_standings(frame, app, tab_content_chunks[1]),
            StandingsFocus::League => render_league_standings(frame, app, tab_content_chunks[1]),
        };
    }
    

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

fn render_standings(frame: &mut Frame, app: &mut App, area: Rect) {
    if let Some(_json) = &app.league_standings {
        match app.standings_type {
            StandingsFocus::WildCard => {
                render_wildcard_standings(frame, app, area);
            }
            StandingsFocus::Division => {
                render_division_standings(frame, app, area);
            }
            StandingsFocus::Conference => {
                render_conference_standings(frame, app, area);
            }
            StandingsFocus::League => {
                render_league_standings(frame, app, area);
            }
        };
    } else {
        // Todo: no data
    }
}



fn render_league_standings(frame: &mut Frame, app: &mut App, area: Rect) {
    // If we are in this function we know that there is data
    let rows = map_league_rows(&app.league_standings.as_ref().unwrap());

    let focused = app.focus == PaneFocus::Content;
    let border_style = if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let header = Row::new([
            "#", "Team", "GP", "W", "L", "OT", "PTS", "P%", "RW", "ROW", "GF", "GA", "DIFF",
            "HOME", "AWAY", "S/O", "L10", "STRK",
        ])
        .style(Style::new().bold())
        .bottom_margin(1);

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

    frame.render_stateful_widget(table, area, &mut app.league_table_state);
}

fn render_conference_standings(frame: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == PaneFocus::Content;
    let border_style = if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let header = Row::new([
            "#", "Team", "GP", "W", "L", "OT", "PTS", "P%", "RW", "ROW", "GF", "GA", "DIFF",
            "HOME", "AWAY", "S/O", "L10", "STRK",
        ])
        .style(Style::new().bold())
        .bottom_margin(1);

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

    // If we are in this function we know that there is data
    let eastern_rows = map_eastern_rows(&app.league_standings.as_ref().unwrap());
    let western_rows = map_western_rows(&app.league_standings.as_ref().unwrap());

    let eastern_table = Table::new(eastern_rows, widths)
        .block(Block::bordered().title(" Eastern Conference Standings ").border_style(border_style))
        .header(header.clone())
        .column_spacing(1)
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    let western_table = Table::new(western_rows, widths)
        .block(Block::bordered().title(" Western Conference Standings ").border_style(border_style))
        .header(header)
        .column_spacing(1)
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    // Render table based on which one is selected
    if app.selected_conference == ConferenceType::Eastern {
        frame.render_stateful_widget(eastern_table, area, &mut app.eastern_table_state);
    }
    else {
        frame.render_stateful_widget(western_table, area, &mut app.western_table_state);
    }   
}

fn render_division_standings(frame: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == PaneFocus::Content;
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

        let rows = map_division_rows(json);
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
            .block(Block::bordered().title(" Division Standings ").border_style(border_style))
            .header(header)
            .column_spacing(1)
            .row_highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(table, area, &mut app.division_table_state);
    } else {
        // Todo: no data
    }
}

fn render_wildcard_standings(frame: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == PaneFocus::Content;
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

        let rows = map_wildcard_rows(json);
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
            .block(Block::bordered().title(" Wildcard Standings ").border_style(border_style))
            .header(header)
            .column_spacing(1)
            .row_highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(table, area, &mut app.wildcard_table_state);
    } else {
        // Todo: no data
    }
}

// Helper functions to map the standings JSON into table rows depending on the standings type
pub fn map_league_rows(json: &str) -> Vec<Row<'static>> {
    let response: StandingsResponse = match serde_json::from_str(json) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Failed to parse standings JSON: {}", e);
            return Vec::new();
        }
    };

    let mut standings = response.standings;
    standings.sort_by_key(|team| team.league_sequence);

    standings
        .iter()
        .map(|team| {
            Row::new(vec![
                team.league_sequence.to_string(),
                team.team_abbrev.default.clone(),
                team.games_played.to_string(),
                team.wins.to_string(),
                team.losses.to_string(),
                team.ot_losses.to_string(),
                team.points.to_string(),
                format!("{:.3}", team.point_pctg),
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

pub fn map_eastern_rows(json: &str) -> Vec<Row<'static>> {
    let response: StandingsResponse = match serde_json::from_str(json) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Failed to parse standings JSON: {}", e);
            return Vec::new();
        }
    };

    let mut east_standings: Vec<_> = response.standings
        .iter()
        .filter(|team| team.conference_abbrev == "E")
        .collect();

    east_standings.sort_by_key(|team| team.conference_sequence);

    east_standings
        .iter()
        .map(|team| {
            Row::new(vec![
                team.conference_sequence.to_string(),
                team.team_abbrev.default.clone(),
                team.games_played.to_string(),
                team.wins.to_string(),
                team.losses.to_string(),
                team.ot_losses.to_string(),
                team.points.to_string(),
                format!("{:.3}", team.point_pctg),
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

pub fn map_western_rows(json: &str) -> Vec<Row<'static>> {
    let response: StandingsResponse = match serde_json::from_str(json) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Failed to parse standings JSON: {}", e);
            return Vec::new();
        }
    };

    let mut east_standings: Vec<_> = response.standings
        .iter()
        .filter(|team| team.conference_abbrev == "W")
        .collect();

    east_standings.sort_by_key(|team| team.conference_sequence);

    east_standings
        .iter()
        .map(|team| {
            Row::new(vec![
                team.conference_sequence.to_string(),
                team.team_abbrev.default.clone(),
                team.games_played.to_string(),
                team.wins.to_string(),
                team.losses.to_string(),
                team.ot_losses.to_string(),
                team.points.to_string(),
                format!("{:.3}", team.point_pctg),
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

pub fn map_wildcard_rows(json: &str) -> Vec<Row<'static>> {
    let response: StandingsResponse = match serde_json::from_str(json) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Failed to parse standings JSON: {}", e);
            return Vec::new();
        }
    };

    let mut standings = response.standings;
    standings.sort_by_key(|team| team.wildcard_sequence);

    standings
        .iter()
        .map(|team| {
            Row::new(vec![
                team.wildcard_sequence.to_string(),
                team.team_abbrev.default.clone(),
                team.games_played.to_string(),
                team.wins.to_string(),
                team.losses.to_string(),
                team.ot_losses.to_string(),
                team.points.to_string(),
                format!("{:.3}", team.point_pctg),
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

pub fn map_division_rows(json: &str) -> Vec<Row<'static>> {
    let response: StandingsResponse = match serde_json::from_str(json) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Failed to parse standings JSON: {}", e);
            return Vec::new();
        }
    };

    let mut standings = response.standings;
    standings.sort_by_key(|team| team.conference_sequence);

    standings
        .iter()
        .map(|team| {
            Row::new(vec![
                team.division_sequence.to_string(),
                team.team_abbrev.default.clone(),
                team.games_played.to_string(),
                team.wins.to_string(),
                team.losses.to_string(),
                team.ot_losses.to_string(),
                team.points.to_string(),
                format!("{:.3}", team.point_pctg),
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
