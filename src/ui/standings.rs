use crate::app::App;
use crate::models::TeamAbbrev;
use crate::models::standings::{Grouping, StandingsResponse, TeamData};
use crate::state::standings_state::StandingsFocus;
use crate::ui::layout::tabs_and_content;
use crate::ui::render::{BORDER_COLOR, border_style};

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Paragraph, Row, Table, TableState, Tabs},
};

const STANDINGS_COLUMNS_NAMES: [&str; 18] = [
    "#", "Team", "GP", "W", "L", "OT", "PTS", "P%", "RW", "ROW", "GF", "GA", "DIFF", "HOME",
    "AWAY", "S/O", "L10", "STRK",
];

const STANDINGS_COLUMN_WIDTHS: [Constraint; 18] = [
    Constraint::Length(3),
    Constraint::Min(24),
    Constraint::Length(3),
    Constraint::Length(3),
    Constraint::Length(3),
    Constraint::Length(3),
    Constraint::Length(5),
    Constraint::Length(7),
    Constraint::Length(4),
    Constraint::Length(4),
    Constraint::Length(5),
    Constraint::Length(5),
    Constraint::Length(5),
    Constraint::Length(9),
    Constraint::Length(9),
    Constraint::Length(5),
    Constraint::Length(9),
    Constraint::Length(5),
];

/// The label shown on a standings tab.
fn tab_label(focus: StandingsFocus) -> &'static str {
    match focus {
        StandingsFocus::WildCard => "Wild Card",
        StandingsFocus::Division => "Division",
        StandingsFocus::Conference => "Conference",
        StandingsFocus::League => "League",
    }
}

pub fn render_standings(frame: &mut Frame, app: &mut App, area: Rect) {
    // Split content chunk into tab + content
    let tab_content_chunks = tabs_and_content(area);

    // Pass visible rows to standings state
    app.state.standings.visible_rows = tab_content_chunks[1].height.saturating_sub(3) as usize;

    // Build the tab list from what the current season actually supports.
    let available = app.state.standings.available_tabs();
    let titles = available
        .iter()
        .map(|t| Line::from(tab_label(*t)))
        .collect::<Vec<_>>();
    let selected_standings_index = available
        .iter()
        .position(|t| *t == app.state.standings.selected_standings)
        .unwrap_or(0);

    let highlight_style = Style::new().fg(BORDER_COLOR).bold().underlined();

    // Season date range (start/end) shown on the right of the tabs border.
    let season_range = app.state.standings.season.as_ref().map(|s| {
        Line::from(format!(
            " start: {}  end: {} ",
            s.standings_start, s.standings_end
        ))
        .style(Style::new().fg(Color::DarkGray))
        .right_aligned()
    });

    let mut block = Block::bordered()
        .border_style(border_style())
        .title(app.state.date_state.format_date_border_title());
    if let Some(range) = season_range {
        block = block.title_top(range);
    }

    let tabs = Tabs::new(titles)
        .select(selected_standings_index)
        .block(block)
        .highlight_style(highlight_style);

    frame.render_widget(tabs, tab_content_chunks[0]);

    if let Some(data) = &app.state.standings.standings_data {
        if data.standings.is_empty() {
            render_message(frame, tab_content_chunks[1], "No standings for this season.");
            return;
        }
        let renderer = StandingsRenderer {
            favorite: app.settings.favorite_team,
        };
        match app.state.standings.selected_standings {
            StandingsFocus::WildCard => {
                if let Some(conf) = app.state.standings.current_conference().cloned() {
                    renderer.render_wildcard(
                        frame,
                        &mut app.state.standings.table_state,
                        &conf,
                        tab_content_chunks[1],
                        data,
                    );
                }
            }
            StandingsFocus::Division => {
                if let Some(div) = app.state.standings.current_division().cloned() {
                    renderer.render_grouping(
                        frame,
                        &mut app.state.standings.table_state,
                        tab_content_chunks[1],
                        data,
                        GroupKind::Division,
                        &div,
                    );
                }
            }
            StandingsFocus::Conference => {
                if let Some(conf) = app.state.standings.current_conference().cloned() {
                    renderer.render_grouping(
                        frame,
                        &mut app.state.standings.table_state,
                        tab_content_chunks[1],
                        data,
                        GroupKind::Conference,
                        &conf,
                    );
                }
            }
            StandingsFocus::League => {
                renderer.render_league(
                    frame,
                    &mut app.state.standings.table_state,
                    tab_content_chunks[1],
                    data,
                );
            }
        };
    } else {
        let message = app
            .state
            .standings
            .out_of_range
            .clone()
            .unwrap_or_else(|| "Loading standings...".to_string());
        render_message(frame, tab_content_chunks[1], &message);
    }
}

fn render_message(frame: &mut Frame, area: Rect, message: &str) {
    let block = Block::bordered()
        .title(" Standings ")
        .border_style(border_style());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line> = message.lines().map(Line::from).collect();

    frame.render_widget(
        Paragraph::new(lines)
            .style(Style::new().fg(Color::DarkGray))
            .centered(),
        inner,
    );
}

/// Which grouping a generic table render targets.
#[derive(Clone, Copy)]
enum GroupKind {
    Division,
    Conference,
}

/// Groups the read-only context needed to render the standings tables so the
/// favorite team doesn't have to be threaded through every function as a parameter.
struct StandingsRenderer {
    favorite: Option<TeamAbbrev>,
}

impl StandingsRenderer {
    fn render_league(
        &self,
        frame: &mut Frame,
        table_state: &mut TableState,
        area: Rect,
        teams: &StandingsResponse,
    ) {
        let rows = self.map_rows(teams, |_| true, |team| team.league_sequence, None);
        let table = create_table(rows, " League Standings ".to_string());
        frame.render_stateful_widget(table, area, table_state);
    }

    /// Render a single division or conference-filtered table
    fn render_grouping(
        &self,
        frame: &mut Frame,
        table_state: &mut TableState,
        area: Rect,
        teams: &StandingsResponse,
        kind: GroupKind,
        group: &Grouping,
    ) {
        let abbrev = group.abbrev.clone();
        let (filter, sort_key, suffix): (
            Box<dyn Fn(&TeamData) -> bool>,
            Box<dyn Fn(&TeamData) -> u8>,
            &str,
        ) = match kind {
            GroupKind::Division => (
                Box::new(move |t: &TeamData| t.division_abbrev.as_deref() == Some(&abbrev)),
                Box::new(|t: &TeamData| t.division_sequence),
                "Division",
            ),
            GroupKind::Conference => (
                Box::new(move |t: &TeamData| t.conference_abbrev.as_deref() == Some(&abbrev)),
                Box::new(|t: &TeamData| t.conference_sequence),
                "Conference",
            ),
        };
        let title = format!(" {} {} Standings ", group.name, suffix);
        self.render_standings_table(frame, table_state, area, teams, filter, sort_key, title);
    }

    fn render_wildcard(
        &self,
        frame: &mut Frame,
        table_state: &mut TableState,
        conference: &Grouping,
        area: Rect,
        teams: &StandingsResponse,
    ) {
        let division_conference_rows_style = Style::new().fg(BORDER_COLOR).underlined();
        let mut rows = Vec::new();

        // Each division within this conference: top 3 teams by division rank.
        for div in teams.divisions_in_conference(&conference.abbrev) {
            rows.push(Row::new(vec!["".to_string(), div.name.clone()]).style(division_conference_rows_style));
            let abbrev = div.abbrev.clone();
            rows.extend(self.map_rows(
                teams,
                move |t| t.division_abbrev.as_deref() == Some(&abbrev),
                |t| t.division_sequence,
                Some(3),
            ));
        }

        // Wildcard teams for this conference (ranked by wildcard sequence).
        rows.push(Row::new(vec!["".to_string(), "Wildcard".to_string()]).style(division_conference_rows_style));
        let conf_abbrev = conference.abbrev.clone();
        rows.extend(self.map_rows(
            teams,
            move |t| t.conference_abbrev.as_deref() == Some(&conf_abbrev) && t.wildcard_sequence != 0,
            |t| t.wildcard_sequence,
            None,
        ));

        let title = format!(" {} Wildcard Standings ", conference.name);
        let table = create_table(rows, title);
        frame.render_stateful_widget(table, area, table_state);
    }

    /// Map the standings data into table rows given a filter for which teams
    /// and how to sort them. The favorite team's row is styled gold.
    fn map_rows<F, S>(
        &self,
        data: &StandingsResponse,
        filter: F,
        sort_key: S,
        n: Option<usize>,
    ) -> Vec<Row<'static>>
    where
        F: Fn(&TeamData) -> bool,
        S: Fn(&TeamData) -> u8,
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
                let is_favorite = self
                    .favorite
                    .is_some_and(|fav| fav == team.team_abbrev.default);
                let row = Row::new(vec![
                    sort_key(team).to_string(),
                    team_name,
                    team.games_played.to_string(),
                    team.wins.to_string(),
                    team.losses.to_string(),
                    team.ot_losses.to_string(),
                    team.points.to_string(),
                    team.point_pctg
                        .map(|v| format!("{:.3}", v))
                        .unwrap_or_else(|| "--".to_string()),
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
                    team.streak_code
                        .as_ref()
                        .zip(team.streak_count.as_ref())
                        .map(|(code, count)| format!("{}{}", code, count))
                        .unwrap_or_else(|| "--".to_string()),
                ]);
                if is_favorite {
                    row.style(Style::new().fg(BORDER_COLOR).bold())
                } else {
                    row
                }
            })
            .collect()
    }

    fn render_standings_table(
        &self,
        frame: &mut Frame,
        table_state: &mut TableState,
        area: Rect,
        teams: &StandingsResponse,
        filter: impl Fn(&TeamData) -> bool,
        sort_key: impl Fn(&TeamData) -> u8,
        title: String,
    ) {
        let rows = self.map_rows(teams, filter, sort_key, None);
        let table = create_table(rows, title);
        frame.render_stateful_widget(table, area, table_state);
    }
}

fn create_table(rows: Vec<Row<'_>>, title: String) -> Table<'_> {
    Table::new(rows, &STANDINGS_COLUMN_WIDTHS)
        .block(Block::bordered().title(title).border_style(border_style()))
        .header(standings_header())
        .column_spacing(1)
        .row_highlight_style(Style::new().bg(Color::DarkGray).bold())
        .highlight_symbol(">> ")
}

/// Helper to create the standings header from the const value
fn standings_header<'a>() -> Row<'a> {
    Row::new(STANDINGS_COLUMNS_NAMES).style(Style::new().bold().add_modifier(Modifier::UNDERLINED))
}
