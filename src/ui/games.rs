use crate::app::App;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Paragraph, Tabs},
};

pub fn render_games(frame: &mut Frame, app: &App, area: Rect) {
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
                .map(|game| {
                    Line::from(format!(
                        "{} @ {}",
                        game.home_team.abbrev, game.away_team.abbrev
                    ))
                })
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

    // Render game information
    if let Some(games_data) = &app.games_data {
        if let Some(game) = games_data.games.get(app.selected_game_index) {
            let home = &game.home_team;
            let away = &game.away_team;

            let lines = vec![
                Line::from(format!("{} vs {}", home.name.default, away.name.default)),
                Line::from(format!("Game state: {}", game.game_state)),
                Line::from(format!("Score: {} - {}", home.score, away.score)),
                Line::from(format!("Shots on Goal: {} - {}", home.sog, away.sog)),
            ];

            let paragraph = Paragraph::new(lines)
                .block(Block::bordered())
                .style(Style::default().fg(Color::White));

            frame.render_widget(paragraph, tab_content_chunks[1]);
        }
    }
}
