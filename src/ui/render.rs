use std::rc::Rc;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Clear, List, ListItem, ListState},
};

use crate::app::App;
use crate::state::app_state::{MenuFocus, PaneFocus};
use crate::ui::{
    date_selector::DateSelectorWidget, games::games, help::HelpWidget,
    input_popup::popup_cursor_position, layout::LayoutAreas, standings,
};

pub const BORDER_FOCUSED_COLOR: Color = Color::Rgb(247, 194, 0); // Orange-yellowish
pub const BORDER_UNFOCUSED_COLOR: Color = Color::DarkGray;

pub fn render(frame: &mut Frame, app: &mut App) {
    match app.state.focus {
        PaneFocus::Help => render_help(frame, frame.area(), app),
        _ => {
            // Split main area into menu + main content
            let content_menu_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(if app.state.display_menu { 15 } else { 0 }), // sidebar
                    Constraint::Percentage(if app.state.display_menu { 85 } else { 100 }), // content
                ])
                .split(frame.area());
            render_menu(frame, app, content_menu_chunks[0]);

            match app.state.selected_menu {
                MenuFocus::Games => games::render_games(frame, app, content_menu_chunks[1]),
                MenuFocus::Standings => {
                    standings::render_standings(frame, app, content_menu_chunks[1])
                }
                MenuFocus::Teams => {}
            }
            if app.state.focus == PaneFocus::DatePicker {
                render_date_picker(frame, app, frame.area());
            }
        }
    }
}

fn render_menu(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.state.focus == PaneFocus::Menu;
    let border_style = if focused {
        Style::default()
            .fg(BORDER_FOCUSED_COLOR)
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
    state.select(Some(app.state.selected_menu.index()));

    frame.render_stateful_widget(list, area, &mut state);
}

fn render_date_picker(f: &mut Frame, app: &mut App, rect: Rect) {
    let chunk = LayoutAreas::create_date_picker(rect);
    f.render_stateful_widget(DateSelectorWidget {}, chunk, &mut app.state.date_state);

    let (cx, cy) = popup_cursor_position(chunk, app.state.date_state.text.len() as u16);
    f.set_cursor_position((cx, cy));
}

fn render_help(frame: &mut Frame, area: Rect, app: &mut App) {
    frame.render_widget(Clear, area);

    // if app.state.show_logs {
    //     draw_border(f, rect, Color::White);
    //     f.render_widget(LogWidget {}, rect);
    //     return;
    // }

    let block = Block::bordered()
        .title(" Help ")
        .border_style(Style::default().fg(BORDER_FOCUSED_COLOR));
    frame.render_widget(block, area);

    frame.render_stateful_widget(HelpWidget {}, area, &mut app.state.help.state);
}

pub fn split_area_horizontal(area: Rect, constraints: impl Into<Vec<Constraint>>) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints.into())
        .split(area)
        .to_vec()
}

pub fn split_area_vertical(area: Rect, constraints: impl Into<Vec<Constraint>>) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints.into())
        .split(area)
}
