use crate::app::{App, ConferenceFocus, DivisionFocus, PaneFocus, StandingsFocus};

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Row, Table, TableState, Tabs},
};

use crate::models::standings::{StandingsResponse, TeamData};

pub fn render_standings(frame: &mut Frame, app: &mut App, area: Rect) {
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

    let focused = app.focus == PaneFocus::Content;
    let border_style = if focused {
        Style::default()
            .fg(Color::LightYellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let highlight_style = if focused {
        Style::default()
            .fg(Color::LightYellow)
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::UNDERLINED)
    } else {
        Style::default()
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::UNDERLINED)
    };

    let tabs = Tabs::new(titles)
        .select(selected_standings)
        .block(Block::bordered().border_style(border_style))
        .highlight_style(highlight_style);

    frame.render_widget(tabs, tab_content_chunks[0]);

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

    if let Some(data) = &app.league_data {
        match app.standings_type {
            StandingsFocus::WildCard => {
                render_wildcard_standings(
                    frame,
                    &mut app.eastern_wildcard_table_state,
                    &mut app.western_wildcard_table_state,
                    &app.selected_wildcard,
                    tab_content_chunks[1],
                    data,
                    border_style,
                    widths,
                    header,
                );
            }
            StandingsFocus::Division => {
                render_division_standings(
                    frame,
                    &mut app.atlantic_table_state,
                    &mut app.metropolitan_table_state,
                    &mut app.central_table_state,
                    &mut app.pacific_table_state,
                    &app.selected_division,
                    tab_content_chunks[1],
                    data,
                    border_style,
                    widths,
                    header,
                );
            }
            StandingsFocus::Conference => {
                render_conference_standings(
                    frame,
                    &mut app.eastern_table_state,
                    &mut app.western_table_state,
                    &app.selected_conference,
                    tab_content_chunks[1],
                    data,
                    border_style,
                    widths,
                    header,
                );
            }
            StandingsFocus::League => {
                render_league_data(
                    frame,
                    &mut app.league_table_state,
                    tab_content_chunks[1],
                    data,
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

pub fn render_league_data(
    frame: &mut Frame,
    table_state: &mut TableState,
    area: Rect,
    teams: &StandingsResponse,
    border_style: Style,
    widths: Vec<Constraint>,
    header: Row<'_>,
) {
    let rows = map_rows(teams, |_| true, |team| team.league_sequence);
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

    frame.render_stateful_widget(table, area, table_state);
}

pub fn render_conference_standings(
    frame: &mut Frame,
    eastern_table_state: &mut TableState,
    western_table_state: &mut TableState,
    selected_conference: &ConferenceFocus,
    area: Rect,
    teams: &StandingsResponse,
    border_style: Style,
    widths: Vec<Constraint>,
    header: Row<'_>,
) {
    let eastern_rows = map_rows(
        teams,
        |team| team.conference_abbrev == "E",
        |team| team.conference_sequence,
    );
    let western_rows = map_rows(
        teams,
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
    match selected_conference {
        ConferenceFocus::Eastern => {
            frame.render_stateful_widget(eastern_table, area, eastern_table_state)
        }
        ConferenceFocus::Western => {
            frame.render_stateful_widget(western_table, area, western_table_state)
        }
    }
}

pub fn render_division_standings(
    frame: &mut Frame,
    atlantic_table_state: &mut TableState,
    metropolitan_table_state: &mut TableState,
    central_table_state: &mut TableState,
    pacific_table_state: &mut TableState,
    selected_division: &DivisionFocus,
    area: Rect,
    teams: &StandingsResponse,
    border_style: Style,
    widths: Vec<Constraint>,
    header: Row<'_>,
) {
    let atlantic_rows = map_rows(
        teams,
        |team| team.division_abbrev == "A",
        |team| team.division_sequence,
    );
    let metropolitan_rows = map_rows(
        teams,
        |team| team.division_abbrev == "M",
        |team| team.division_sequence,
    );
    let central_rows = map_rows(
        teams,
        |team| team.division_abbrev == "C",
        |team| team.division_sequence,
    );
    let pacific_rows = map_rows(
        teams,
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
    match selected_division {
        DivisionFocus::Atlantic => {
            frame.render_stateful_widget(atlantic_table, area, atlantic_table_state)
        }
        DivisionFocus::Metropolitan => {
            frame.render_stateful_widget(metropolitan_table, area, metropolitan_table_state)
        }
        DivisionFocus::Central => {
            frame.render_stateful_widget(central_table, area, central_table_state)
        }
        DivisionFocus::Pacific => {
            frame.render_stateful_widget(pacific_table, area, pacific_table_state)
        }
    }
}

pub fn render_wildcard_standings(
    frame: &mut Frame,
    eastern_wildcard_table_state: &mut TableState,
    western_wildcard_table_state: &mut TableState,
    selected_wildcard: &ConferenceFocus,
    area: Rect,
    teams: &StandingsResponse,
    border_style: Style,
    widths: Vec<Constraint>,
    header: Row<'_>,
) {
    // Eastern wildcard
    let top_3_atlantic: Vec<Row> = map_rows(
        teams,
        |team| team.division_abbrev == "A",
        |team| team.division_sequence,
    )
    .into_iter()
    .take(3)
    .collect();
    let top_3_metropolitan: Vec<Row> = map_rows(
        teams,
        |team| team.division_abbrev == "M",
        |team| team.division_sequence,
    )
    .into_iter()
    .take(3)
    .collect();
    let eastern_wildcard = map_rows(
        teams,
        |team| team.conference_abbrev == "E" && team.wildcard_sequence != 0,
        |team| team.wildcard_sequence,
    );
    let mut eastern_rows = Vec::new();
    eastern_rows.extend(top_3_atlantic);
    eastern_rows.extend(top_3_metropolitan);
    eastern_rows.extend(eastern_wildcard);

    // Western wildcard
    let top_3_central: Vec<Row> = map_rows(
        teams,
        |team| team.division_abbrev == "C",
        |team| team.division_sequence,
    )
    .into_iter()
    .take(3)
    .collect();
    let top_3_pacific: Vec<Row> = map_rows(
        teams,
        |team| team.division_abbrev == "P",
        |team| team.division_sequence,
    )
    .into_iter()
    .take(3)
    .collect();
    let western_wildcard = map_rows(
        teams,
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
    match selected_wildcard {
        ConferenceFocus::Eastern => {
            frame.render_stateful_widget(eastern_wildcard_table, area, eastern_wildcard_table_state)
        }
        ConferenceFocus::Western => {
            frame.render_stateful_widget(western_wildcard_table, area, western_wildcard_table_state)
        }
    }
}

// Helper functions to map the standings JSON into table rows given a filter for which teams and how to sort them
pub fn map_rows<F, S>(data: &StandingsResponse, filter: F, sort_key: S) -> Vec<Row<'static>>
where
    F: Fn(&TeamData) -> bool,
    S: Fn(&TeamData) -> u32,
{
    let mut standings: Vec<_> = data.standings.iter().filter(|team| filter(team)).collect();

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
