use crate::app::App;
use crate::models::boxscore::{
    BoxscoreResponse, Defenseman, Forward, Goalie, PlayerData, Position,
};
use crate::state::app_state::PaneFocus;
use crate::state::games_state::{BoxscorePosition, BoxscoreTeam};

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Row, Table},
};

const BOXSCORE_FORWARDS_COLUMNS: [&str; 17] = [
    "#", "Forwards", "POS", "G", "A", "P", "+/-", "PIM", "TOI", "SHIFT", "PPG", "S", "BLK", "HITS",
    "GV", "TK", "FO%",
];
const BOXSCORE_DEFENSE_COLUMNS: [&str; 15] = [
    "#",
    "Defensemen",
    "G",
    "A",
    "P",
    "+/-",
    "PIM",
    "TOI",
    "SHIFT",
    "PPG",
    "S",
    "BLK",
    "HITS",
    "GV",
    "TK",
];
const BOXSCORE_GOALIES_COLUMNS: [&str; 10] = [
    "#", "Goalies", "SA", "SV", "GA", "EV", "PP", "SH", "SV%", "TOI",
];

const BOXSCORE_FORWARDS_COLUMN_WIDTHS: [Constraint; 17] = [
    Constraint::Length(3),
    Constraint::Min(20),
    Constraint::Length(4),
    Constraint::Length(3),
    Constraint::Length(3),
    Constraint::Length(3),
    Constraint::Length(4),
    Constraint::Length(4),
    Constraint::Length(7),
    Constraint::Length(6),
    Constraint::Length(4),
    Constraint::Length(3),
    Constraint::Length(5),
    Constraint::Length(6),
    Constraint::Length(4),
    Constraint::Length(4),
    Constraint::Length(5),
];

const BOXSCORE_DEFENSE_COLUMN_WIDTHS: [Constraint; 15] = [
    Constraint::Length(3),
    Constraint::Min(20),
    Constraint::Length(3),
    Constraint::Length(3),
    Constraint::Length(3),
    Constraint::Length(4),
    Constraint::Length(4),
    Constraint::Length(7),
    Constraint::Length(6),
    Constraint::Length(4),
    Constraint::Length(3),
    Constraint::Length(5),
    Constraint::Length(6),
    Constraint::Length(4),
    Constraint::Length(4),
];

const BOXSCORE_GOALIES_COLUMN_WIDTHS: [Constraint; 10] = [
    Constraint::Length(3),
    Constraint::Min(20),
    Constraint::Length(4),
    Constraint::Length(4),
    Constraint::Length(4),
    Constraint::Length(6),
    Constraint::Length(5),
    Constraint::Length(5),
    Constraint::Length(5),
    Constraint::Length(6),
];

pub fn render_boxscore(frame: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.state.focus == PaneFocus::Content;
    let border_style = if focused {
        Style::default()
            .fg(Color::Rgb(247, 194, 0))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let is_home = matches!(&app.state.games.boxscore_selected_team, BoxscoreTeam::Home);
    let boxscore = app
        .state
        .games
        .games_data
        .as_ref()
        .and_then(|g| g.games.get(app.state.games.selected_game_index))
        .and_then(|g| app.state.games.boxscore_data.get(&g.id));

    if let Some(boxscore) = boxscore {
        let rows = map_rows(
            boxscore,
            is_home,
            &app.state.games.boxscore_selected_position,
        );
        let (widths, header): (&[Constraint], &[&str]) =
            match &app.state.games.boxscore_selected_position {
                BoxscorePosition::Forwards => {
                    (&BOXSCORE_FORWARDS_COLUMN_WIDTHS, &BOXSCORE_FORWARDS_COLUMNS)
                }
                BoxscorePosition::Defensemen => {
                    (&BOXSCORE_DEFENSE_COLUMN_WIDTHS, &BOXSCORE_DEFENSE_COLUMNS)
                }
                BoxscorePosition::Goalies => {
                    (&BOXSCORE_GOALIES_COLUMN_WIDTHS, &BOXSCORE_GOALIES_COLUMNS)
                }
            };
        let table = create_table(
            rows,
            get_boxscore_title(is_home, boxscore),
            border_style,
            widths,
            header,
        );
        frame.render_stateful_widget(table, area, &mut app.state.games.boxscore_table_state);
    } else {
        // Todo
    }
}

fn map_forwards_rows(players: &[Forward]) -> Vec<Row<'static>> {
    players
        .iter()
        .map(|p| {
            let name = p.name.default.clone();
            let pos = match p.position {
                Position::LeftWing => "LW",
                Position::RightWing => "RW",
                Position::Center => "C",
                _ => "?", // Unexpected
            };
            Row::new(vec![
                p.sweater_number.to_string(),
                name,
                pos.to_string(),
                p.goals.to_string(),
                p.assists.to_string(),
                p.points.to_string(),
                p.plus_minus.to_string(),
                p.pim.unwrap_or(0).to_string(),
                p.toi.as_ref().map_or("--".to_string(), |t| t.clone()),
                p.shifts.to_string(),
                p.power_play_goals.to_string(),
                p.sog.to_string(),
                p.blocked_shots.to_string(),
                p.hits.to_string(),
                p.giveaways.to_string(),
                p.takeaways.to_string(),
                p.faceoff_winning_pctg
                    .filter(|f| *f > 0.0)
                    .map(|f| format!("{:.1}", f * 100.0))
                    .unwrap_or_else(|| "--".to_string()),
            ])
        })
        .collect()
}

fn map_defensemen_rows(players: &[Defenseman]) -> Vec<Row<'static>> {
    players
        .iter()
        .map(|p| {
            let name = p.name.default.clone();
            Row::new(vec![
                p.sweater_number.to_string(),
                name,
                p.goals.to_string(),
                p.assists.to_string(),
                p.points.to_string(),
                p.plus_minus.to_string(),
                p.pim.unwrap_or(0).to_string(),
                p.toi.as_ref().map_or("--".to_string(), |t| t.clone()),
                p.shifts.to_string(),
                p.power_play_goals.to_string(),
                p.sog.to_string(),
                p.blocked_shots.to_string(),
                p.hits.to_string(),
                p.giveaways.to_string(),
                p.takeaways.to_string(),
            ])
        })
        .collect()
}

fn map_goalie_rows(players: &[Goalie]) -> Vec<Row<'static>> {
    players
        .iter()
        .map(|p| {
            Row::new(vec![
                p.sweater_number.to_string(),
                p.name.default.clone(),
                p.shots_against.to_string(),
                p.saves.to_string(),
                p.goals_against.to_string(),
                p.even_strength_shots_against.clone(),
                p.power_play_shots_against.clone(),
                p.shorthanded_shots_against.clone(),
                p.save_pctg
                    .map(|f| format!("{:.4}", f))
                    .unwrap_or_else(|| "--".to_string()),
                p.toi.as_ref().map_or("--".to_string(), |t| t.clone()),
            ])
        })
        .collect()
}

fn get_player_data(data: &BoxscoreResponse, is_home: bool) -> Option<&PlayerData> {
    data.player_by_game_stats
        .as_ref()
        .map(|s| if is_home { &s.home_team } else { &s.away_team })
}

pub fn map_rows(
    data: &BoxscoreResponse,
    is_home: bool,
    position: &BoxscorePosition,
) -> Vec<Row<'static>> {
    let Some(players) = get_player_data(data, is_home) else {
        return vec![];
    };

    match position {
        BoxscorePosition::Forwards => map_forwards_rows(&players.forwards),
        BoxscorePosition::Defensemen => map_defensemen_rows(&players.defense),
        BoxscorePosition::Goalies => map_goalie_rows(&players.goalies),
    }
}

fn create_table<'a>(
    rows: Vec<Row<'a>>,
    title: String,
    border_style: Style,
    widths: &[Constraint],
    header: &[&str],
) -> Table<'a> {
    Table::new(rows, widths)
        .block(Block::bordered().title(title).border_style(border_style))
        .header(
            Row::new(header.iter().map(|s| s.to_string()).collect::<Vec<_>>())
                .style(Style::new().bold())
                .bottom_margin(1),
        )
        .column_spacing(1)
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ")
}

fn get_boxscore_title(is_home: bool, boxscore: &BoxscoreResponse) -> String {
    if is_home {
        " ".to_string() + &boxscore.home_team.common_name.default + " "
    } else {
        " ".to_string() + &boxscore.away_team.common_name.default + " "
    }
}
