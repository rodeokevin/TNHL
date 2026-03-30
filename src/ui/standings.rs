use crate::app::App;
use crate::state::app_state::PaneFocus;
use crate::state::standings_state::{ConferenceFocus, DivisionFocus, StandingsFocus};

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Row, Table, TableState, Tabs},
};

const STANDINGS_COLUMNS: [&str; 18] = [
    "#", "Team", "GP", "W", "L", "OT", "PTS", "P%", "RW", "ROW", "GF", "GA", "DIFF", "HOME",
    "AWAY", "S/O", "L10", "STRK",
];

const STANDINGS_COLUMN_WIDTHS: [Constraint; 18] = [
    Constraint::Length(3),
    Constraint::Min(23),
    Constraint::Length(4),
    Constraint::Length(4),
    Constraint::Length(4),
    Constraint::Length(4),
    Constraint::Length(5),
    Constraint::Length(7),
    Constraint::Length(4),
    Constraint::Length(4),
    Constraint::Length(5),
    Constraint::Length(5),
    Constraint::Length(5),
    Constraint::Length(11),
    Constraint::Length(11),
    Constraint::Length(6),
    Constraint::Length(9),
    Constraint::Length(6),
];

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

    let titles = ["Wild Card", "Division", "Conference", "League"]
        .iter()
        .map(|t| Line::from(*t))
        .collect::<Vec<_>>();

    let selected_standings = match app.state.standings.focus {
        StandingsFocus::WildCard => 0,
        StandingsFocus::Division => 1,
        StandingsFocus::Conference => 2,
        StandingsFocus::League => 3,
    };

    let focused = app.state.focus == PaneFocus::Content;
    let border_style = if focused {
        Style::default()
            .fg(Color::Rgb(247, 194, 0))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let highlight_style = if focused {
        Style::default()
            .fg(Color::Rgb(247, 194, 0))
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::UNDERLINED)
    } else {
        Style::default()
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::UNDERLINED)
    };

    let tabs = Tabs::new(titles)
        .select(selected_standings)
        .block(
            Block::bordered()
                .border_style(border_style)
                .title(app.state.date_selector.format_date_border_title()),
        )
        .highlight_style(highlight_style);

    frame.render_widget(tabs, tab_content_chunks[0]);

    if let Some(data) = &app.state.league_data {
        match app.state.standings.focus {
            StandingsFocus::WildCard => {
                render_wildcard_standings(
                    frame,
                    &mut app.state.standings.eastern_wildcard_table_state,
                    &mut app.state.standings.western_wildcard_table_state,
                    &app.state.standings.selected_wildcard,
                    tab_content_chunks[1],
                    data,
                    border_style,
                );
            }
            StandingsFocus::Division => {
                render_division_standings(
                    frame,
                    &mut app.state.standings.atlantic_table_state,
                    &mut app.state.standings.metropolitan_table_state,
                    &mut app.state.standings.central_table_state,
                    &mut app.state.standings.pacific_table_state,
                    &app.state.standings.selected_division,
                    tab_content_chunks[1],
                    data,
                    border_style,
                );
            }
            StandingsFocus::Conference => {
                render_conference_standings(
                    frame,
                    &mut app.state.standings.eastern_table_state,
                    &mut app.state.standings.western_table_state,
                    &app.state.standings.selected_conference,
                    tab_content_chunks[1],
                    data,
                    border_style,
                );
            }
            StandingsFocus::League => {
                render_league_standings(
                    frame,
                    &mut app.state.standings.league_table_state,
                    tab_content_chunks[1],
                    data,
                    border_style,
                );
            }
        };
    } else {
        // Todo: no data
    }
}

fn render_league_standings(
    frame: &mut Frame,
    table_state: &mut TableState,
    area: Rect,
    teams: &StandingsResponse,
    border_style: Style,
) {
    let rows = map_rows(teams, |_| true, |team| team.league_sequence, None);
    let table = create_table(rows, " League Standings ".to_string(), border_style);
    frame.render_stateful_widget(table, area, table_state);
}

fn render_conference_standings(
    frame: &mut Frame,
    eastern_table_state: &mut TableState,
    western_table_state: &mut TableState,
    selected_conference: &ConferenceFocus,
    area: Rect,
    teams: &StandingsResponse,
    border_style: Style,
) {
    let (table_state, abbrev, title) = match selected_conference {
        ConferenceFocus::Eastern => (eastern_table_state, "E", " Eastern Conference Standings "),
        ConferenceFocus::Western => (western_table_state, "W", " Western Conference Standings "),
    };

    render_standings_table(
        frame,
        table_state,
        area,
        teams,
        |team| team.conference_abbrev == abbrev,
        |team| team.conference_sequence,
        title.to_string(),
        border_style,
    );
}

fn render_division_standings(
    frame: &mut Frame,
    atlantic_table_state: &mut TableState,
    metropolitan_table_state: &mut TableState,
    central_table_state: &mut TableState,
    pacific_table_state: &mut TableState,
    selected_division: &DivisionFocus,
    area: Rect,
    teams: &StandingsResponse,
    border_style: Style,
) {
    let (table_state, abbrev, title) = match selected_division {
        DivisionFocus::Atlantic => (atlantic_table_state, "A", " Atlantic Division Standings "),
        DivisionFocus::Metropolitan => (
            metropolitan_table_state,
            "M",
            " Metropolitan Division Standings ",
        ),
        DivisionFocus::Central => (central_table_state, "C", " Central Division Standings "),
        DivisionFocus::Pacific => (pacific_table_state, "P", " Pacific Division Standings "),
    };

    render_standings_table(
        frame,
        table_state,
        area,
        teams,
        |team| team.division_abbrev == abbrev,
        |team| team.division_sequence,
        title.to_string(),
        border_style,
    );
}

fn render_wildcard_standings(
    frame: &mut Frame,
    eastern_wildcard_table_state: &mut TableState,
    western_wildcard_table_state: &mut TableState,
    selected_wildcard: &ConferenceFocus,
    area: Rect,
    teams: &StandingsResponse,
    border_style: Style,
) {
    let (table_state, div1_abbr, div1_full, div2_abbr, div2_full, conf, title) =
        match selected_wildcard {
            ConferenceFocus::Eastern => (
                eastern_wildcard_table_state,
                "A",
                "Atlantic",
                "M",
                "Metropolitan",
                "E",
                " Eastern Wildcard Standings ",
            ),
            ConferenceFocus::Western => (
                western_wildcard_table_state,
                "C",
                "Central",
                "P",
                "Pacific",
                "W",
                " Western Wildcard Standings ",
            ),
        };
    let division_conference_rows_style = Style::default()
        .fg(Color::Blue)
        .add_modifier(Modifier::UNDERLINED);
    let mut rows = Vec::new();
    rows.extend(vec![
        Row::new(vec!["", div1_full]).style(division_conference_rows_style),
    ]);
    rows.extend(map_rows(
        teams,
        |t| t.division_abbrev == div1_abbr,
        |t| t.division_sequence,
        Some(3),
    ));
    rows.extend(vec![
        Row::new(vec!["", div2_full]).style(division_conference_rows_style),
    ]);
    rows.extend(map_rows(
        teams,
        |t| t.division_abbrev == div2_abbr,
        |t| t.division_sequence,
        Some(3),
    ));
    rows.extend(vec![
        Row::new(vec!["", "Wildcard"]).style(division_conference_rows_style),
    ]);
    rows.extend(map_rows(
        teams,
        |t| t.conference_abbrev == conf && t.wildcard_sequence != 0,
        |t| t.wildcard_sequence,
        None,
    ));

    let table = create_table(rows, title.to_string(), border_style);
    frame.render_stateful_widget(table, area, table_state);
}

fn create_table(rows: Vec<Row<'_>>, title: String, border_style: Style) -> Table<'_> {
    Table::new(rows, &STANDINGS_COLUMN_WIDTHS)
        .block(Block::bordered().title(title).border_style(border_style))
        .header(standings_header())
        .column_spacing(1)
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ")
}

// Helper to create the standings header from the const value
fn standings_header<'a>() -> Row<'a> {
    Row::new(STANDINGS_COLUMNS)
        .style(Style::new().bold())
        .bottom_margin(1)
}

// Helper functions to map the standings JSON into table rows given a filter for which teams and how to sort them
fn map_rows<F, S>(
    data: &StandingsResponse,
    filter: F,
    sort_key: S,
    n: Option<usize>,
) -> Vec<Row<'static>>
where
    F: Fn(&TeamData) -> bool,
    S: Fn(&TeamData) -> u32,
{
    let mut standings: Vec<_> = data.standings.iter().filter(|team| filter(team)).collect();

    standings.sort_by_key(|team| sort_key(team));

    standings
        .into_iter()
        .take(n.unwrap_or(usize::MAX)) // take all entries if n is not specified
        .map(|team| {
            let team_name = if let Some(indicator) = &team.clinch_indicator {
                team.team_name.default.clone() + " - " + indicator
            } else {
                team.team_name.default.clone()
            };
            Row::new(vec![
                sort_key(team).to_string(),
                team_name,
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

// Helper to render standings
fn render_standings_table(
    frame: &mut Frame,
    table_state: &mut TableState,
    area: Rect,
    teams: &StandingsResponse,
    filter: impl Fn(&TeamData) -> bool,
    sort_key: impl Fn(&TeamData) -> u32,
    title: String,
    border_style: Style,
) {
    let rows = map_rows(teams, filter, sort_key, None);
    let table = create_table(rows, title, border_style);
    frame.render_stateful_widget(table, area, table_state);
}
