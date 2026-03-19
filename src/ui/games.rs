use crate::app::App;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Tabs},
};

use crate::models::games::{GameData, GamesResponse};

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
