pub mod standings;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, List, ListItem, ListState, Paragraph, Tabs},
};

use crate::app::{App, MenuFocus, PaneFocus};

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
        MenuFocus::Standings => standings::render_standings(frame, app, content_menu_chunks[1]),
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
