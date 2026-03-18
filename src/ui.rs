use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, List, ListItem, ListState, Paragraph, Row, Table, Tabs},
};

use serde::Deserialize;

use crate::app::{
    App, ConferenceType, DivisionType, GamesResponse, MenuFocus, PaneFocus, StandingsFocus,
};

#[derive(Debug, Deserialize)]
struct StandingsResponse {
    standings: Vec<TeamData>,
}

#[derive(Debug, Deserialize)]
pub struct TeamData {
    #[serde(rename = "teamName")]
    pub team_name: TeamName,
    #[serde(rename = "teamAbbrev")]
    pub team_abbrev: TeamAbbrev,
    #[serde(rename = "conferenceAbbrev")]
    pub conference_abbrev: String,
    #[serde(rename = "divisionAbbrev")]
    pub division_abbrev: String,
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

    match app.selected_menu {
        MenuFocus::Games => render_games(frame, app, content_menu_chunks[1]),
        MenuFocus::Standings => render_standings(frame, app, content_menu_chunks[1]),
        MenuFocus::Teams => {}
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
        ListItem::new("Games"),
        ListItem::new("Standings"),
        ListItem::new("Teams"),
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
    state.select(Some(app.selected_menu.index()));

    frame.render_stateful_widget(list, area, &mut state);
}

fn render_standings(frame: &mut Frame, app: &mut App, area: Rect) {
    // Split content chunk into tab + content
    let tab_content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // tabs
            Constraint::Min(1),    // table
        ])
        .split(area);

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

    let focused = app.focus == PaneFocus::Content;
    let border_style = if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let header = Row::new([
        "#", "Team", "GP", "W", "L", "OT", "PTS", "P%", "RW", "ROW", "GF", "GA", "DIFF", "HOME",
        "AWAY", "S/O", "L10", "STRK",
    ])
    .style(Style::new().bold())
    .bottom_margin(1);

    let widths = vec![
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

    if let Some(_json) = &app.league_standings {
        match app.standings_type {
            StandingsFocus::WildCard => {
                render_wildcard_standings(
                    frame,
                    app,
                    tab_content_chunks[1],
                    border_style,
                    widths,
                    header,
                );
            }
            StandingsFocus::Division => {
                render_division_standings(
                    frame,
                    app,
                    tab_content_chunks[1],
                    border_style,
                    widths,
                    header,
                );
            }
            StandingsFocus::Conference => {
                render_conference_standings(
                    frame,
                    app,
                    tab_content_chunks[1],
                    border_style,
                    widths,
                    header,
                );
            }
            StandingsFocus::League => {
                render_league_standings(
                    frame,
                    app,
                    tab_content_chunks[1],
                    border_style,
                    widths,
                    header,
                );
            }
        };
    } else {
        // Todo: no data
    }
}

fn render_league_standings(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    border_style: Style,
    widths: Vec<Constraint>,
    header: Row<'_>,
) {
    // If we are in this function we know that there is data
    let rows = map_rows(
        &app.league_standings.as_deref().unwrap(),
        |_| true,
        |team| team.league_sequence,
    );

    let table = Table::new(rows, widths)
        .block(
            Block::bordered()
                .title(" League Standings ")
                .border_style(border_style),
        )
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

fn render_conference_standings(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    border_style: Style,
    widths: Vec<Constraint>,
    header: Row<'_>,
) {
    // If we are in this function we know that there is data
    let eastern_rows = map_rows(
        &app.league_standings.as_deref().unwrap(),
        |team| team.conference_abbrev == "E",
        |team| team.conference_sequence,
    );
    let western_rows = map_rows(
        &app.league_standings.as_deref().unwrap(),
        |team| team.conference_abbrev == "W",
        |team| team.conference_sequence,
    );

    let eastern_table = Table::new(eastern_rows, widths.clone())
        .block(
            Block::bordered()
                .title(" Eastern Conference Standings ")
                .border_style(border_style),
        )
        .header(header.clone())
        .column_spacing(1)
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    let western_table = Table::new(western_rows, widths)
        .block(
            Block::bordered()
                .title(" Western Conference Standings ")
                .border_style(border_style),
        )
        .header(header)
        .column_spacing(1)
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    // Render based on which conference is selected
    match app.selected_conference {
        ConferenceType::Eastern => {
            frame.render_stateful_widget(eastern_table, area, &mut app.eastern_table_state)
        }
        ConferenceType::Western => {
            frame.render_stateful_widget(western_table, area, &mut app.western_table_state)
        }
    }
}

fn render_division_standings(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    border_style: Style,
    widths: Vec<Constraint>,
    header: Row<'_>,
) {
    let atlantic_rows = map_rows(
        &app.league_standings.as_deref().unwrap(),
        |team| team.division_abbrev == "A",
        |team| team.division_sequence,
    );
    let metropolitan_rows = map_rows(
        &app.league_standings.as_deref().unwrap(),
        |team| team.division_abbrev == "M",
        |team| team.division_sequence,
    );
    let central_rows = map_rows(
        &app.league_standings.as_deref().unwrap(),
        |team| team.division_abbrev == "C",
        |team| team.division_sequence,
    );
    let pacific_rows = map_rows(
        &app.league_standings.as_deref().unwrap(),
        |team| team.division_abbrev == "P",
        |team| team.division_sequence,
    );

    let atlantic_table = Table::new(atlantic_rows, widths.clone())
        .block(
            Block::bordered()
                .title(" Atlantic Division Standings ")
                .border_style(border_style),
        )
        .header(header.clone())
        .column_spacing(1)
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    let metropolitan_table = Table::new(metropolitan_rows, widths.clone())
        .block(
            Block::bordered()
                .title(" Metropolitan Division Standings ")
                .border_style(border_style),
        )
        .header(header.clone())
        .column_spacing(1)
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    let central_table = Table::new(central_rows, widths.clone())
        .block(
            Block::bordered()
                .title(" Central Division Standings ")
                .border_style(border_style),
        )
        .header(header.clone())
        .column_spacing(1)
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    let pacific_table = Table::new(pacific_rows, widths)
        .block(
            Block::bordered()
                .title(" Pacific Division Standings ")
                .border_style(border_style),
        )
        .header(header)
        .column_spacing(1)
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // Render based on which division is selected
    match app.selected_division {
        DivisionType::Atlantic => {
            frame.render_stateful_widget(atlantic_table, area, &mut app.atlantic_table_state)
        }
        DivisionType::Metropolitan => frame.render_stateful_widget(
            metropolitan_table,
            area,
            &mut app.metropolitan_table_state,
        ),
        DivisionType::Central => {
            frame.render_stateful_widget(central_table, area, &mut app.central_table_state)
        }
        DivisionType::Pacific => {
            frame.render_stateful_widget(pacific_table, area, &mut app.pacific_table_state)
        }
    }
}

fn render_wildcard_standings(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    border_style: Style,
    widths: Vec<Constraint>,
    header: Row<'_>,
) {
    // If we are in this function we know that there is data
    // Eastern wildcard
    let top_3_atlantic: Vec<Row> = map_rows(
        &app.league_standings.as_deref().unwrap(),
        |team| team.division_abbrev == "A",
        |team| team.division_sequence,
    )
    .into_iter()
    .take(3)
    .collect();
    let top_3_metropolitan: Vec<Row> = map_rows(
        &app.league_standings.as_deref().unwrap(),
        |team| team.division_abbrev == "M",
        |team| team.division_sequence,
    )
    .into_iter()
    .take(3)
    .collect();
    let eastern_wildcard = map_rows(
        &app.league_standings.as_deref().unwrap(),
        |team| team.conference_abbrev == "E" && team.wildcard_sequence != 0,
        |team| team.wildcard_sequence,
    );
    let mut eastern_rows = Vec::new();
    eastern_rows.extend(top_3_atlantic);
    eastern_rows.extend(top_3_metropolitan);
    eastern_rows.extend(eastern_wildcard);

    // Western wildcard
    let top_3_central: Vec<Row> = map_rows(
        &app.league_standings.as_deref().unwrap(),
        |team| team.division_abbrev == "C",
        |team| team.division_sequence,
    )
    .into_iter()
    .take(3)
    .collect();
    let top_3_pacific: Vec<Row> = map_rows(
        &app.league_standings.as_deref().unwrap(),
        |team| team.division_abbrev == "P",
        |team| team.division_sequence,
    )
    .into_iter()
    .take(3)
    .collect();
    let western_wildcard = map_rows(
        &app.league_standings.as_deref().unwrap(),
        |team| team.conference_abbrev == "W" && team.wildcard_sequence != 0,
        |team| team.wildcard_sequence,
    );
    let mut western_rows = Vec::new();
    western_rows.extend(top_3_central);
    western_rows.extend(top_3_pacific);
    western_rows.extend(western_wildcard);

    let eastern_wildcard_table = Table::new(eastern_rows, widths.clone())
        .block(
            Block::bordered()
                .title(" Eastern Wildcard Standings ")
                .border_style(border_style),
        )
        .header(header.clone())
        .column_spacing(1)
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    let western_wildcard_table = Table::new(western_rows, widths)
        .block(
            Block::bordered()
                .title(" Western Wildcard Standings ")
                .border_style(border_style),
        )
        .header(header)
        .column_spacing(1)
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // Render based on which wildcard is selected
    match app.selected_wildcard {
        ConferenceType::Eastern => frame.render_stateful_widget(
            eastern_wildcard_table,
            area,
            &mut app.eastern_wildcard_table_state,
        ),
        ConferenceType::Western => frame.render_stateful_widget(
            western_wildcard_table,
            area,
            &mut app.western_wildcard_table_state,
        ),
    }
}

// Helper functions to map the standings JSON into table rows given a filter for which teams and how to sort them
pub fn map_rows<F, S>(json: &str, filter: F, sort_key: S) -> Vec<Row<'static>>
where
    F: Fn(&TeamData) -> bool,
    S: Fn(&TeamData) -> u32,
{
    let response: StandingsResponse = match serde_json::from_str(json) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Failed to parse standings JSON: {}", e);
            return Vec::new();
        }
    };

    let mut standings: Vec<_> = response
        .standings
        .iter()
        .filter(|team| filter(team))
        .collect();
    standings.sort_by_key(|team| sort_key(team));

    standings
        .iter()
        .map(|team| {
            Row::new(vec![
                sort_key(team).to_string(),
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

pub fn render_games(frame: &mut Frame, app: &mut App, area: Rect) {
    // Split content chunk into tab + content
    let tab_content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // tabs
            Constraint::Min(1),    // table
        ])
        .split(area);

    let titles: Vec<Line> = app
        .games_data
        .as_ref()
        .map(|data| {
            data.games
                .iter()
                .map(|game| Line::from(format!("{}", game.id,)))
                .collect()
        })
        .unwrap_or_default();

    let tabs = Tabs::new(titles)
        .select(app.selected_game_index)
        .block(Block::bordered())
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, tab_content_chunks[0]);
}
