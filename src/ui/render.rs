use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Clear, List, ListItem, ListState, Paragraph},
};

use crate::state::{
    app_state::{MenuFocus, PaneFocus},
    games_state::GamesFocus,
    playoffs_state::PlayoffsFocus,
};
use crate::ui::{
    date_selector::DateSelectorWidget, games::games, help::HelpWidget,
    input_popup::popup_cursor_position, layout::LayoutAreas, playoffs, standings,
    team_stats::team_selector::TeamSelectorWidget, year_selector::YearSelectorWidget,
};
use crate::{app::App, ui::team_stats::team_stats};

pub const BORDER_COLOR: Color = Color::Rgb(247, 194, 0); // Orange-yellowish

const MENU_WIDTH: u16 = 19;

pub fn border_style() -> Style {
    Style::new().fg(BORDER_COLOR).bold()
}

pub fn render(frame: &mut Frame, app: &mut App) {
    match app.state.focus {
        PaneFocus::Help => render_help(frame, frame.area(), app),
        _ => {
            // Split main area into menu + main content. The menu uses a fixed
            // width so context hint text isn't cut off.
            let content_menu_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(if app.state.display_menu {
                        MENU_WIDTH
                    } else {
                        0
                    }), // menu
                    Constraint::Min(0), // content
                ])
                .split(frame.area());
            render_menu(frame, app, content_menu_chunks[0]);

            match app.state.selected_menu {
                MenuFocus::Games => games::render_games(frame, app, content_menu_chunks[1]),
                MenuFocus::Standings => {
                    standings::render_standings(frame, app, content_menu_chunks[1]);
                }
                MenuFocus::TeamStats => {
                    team_stats::render_team_stats(frame, app, content_menu_chunks[1]);
                    if app.state.focus == PaneFocus::TeamPicker {
                        render_team_picker(frame, app, frame.area());
                    }
                }
                MenuFocus::Playoffs => match app.state.playoffs.focus {
                    PlayoffsFocus::Bracket => {
                        playoffs::bracket::render_playoffs(frame, app, content_menu_chunks[1])
                    }
                    PlayoffsFocus::Series => {
                        playoffs::series::render_series(frame, app, content_menu_chunks[1])
                    }
                },
            }
            if app.state.focus == PaneFocus::DatePicker {
                render_date_picker(frame, app, frame.area());
            };
        }
    }
}

fn render_menu(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::bordered()
        .title(" Menu ")
        .border_style(border_style());

    frame.render_widget(block.clone(), area);

    let inner = block.inner(area);

    // Context-sensitive key hints shown at the bottom of the menu panel,
    // just above the "Help: ?" line. Each entry is a "Label  key" pair.
    let hints: Vec<(&str, &str)> = context_hints(app);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(4),                     // menu list
            Constraint::Length(hints.len() as u16), // context hints
            Constraint::Length(1),                  // "Help: ?"
        ])
        .split(inner);

    let menu_items = vec![
        ListItem::new("1.Games"),
        ListItem::new("2.Standings"),
        ListItem::new("3.Team Stats"),
        ListItem::new("4.Playoffs"),
    ];

    let list = List::new(menu_items)
        .highlight_style(Style::new().bg(Color::DarkGray).bold())
        .highlight_symbol(">> ");

    let mut state = ListState::default();
    state.select(Some(app.state.selected_menu.index()));

    frame.render_stateful_widget(list, chunks[0], &mut state);

    if !hints.is_empty() {
        let hint_lines: Vec<Line> = hints
            .iter()
            .map(|(label, key)| Line::from(format!("{label}: {key}")))
            .collect();
        let hints_widget = Paragraph::new(hint_lines).style(Style::new().fg(Color::DarkGray));
        frame.render_widget(hints_widget, chunks[1]);
    }

    let help = Paragraph::new("Help: ?").style(Style::new().fg(Color::DarkGray));

    frame.render_widget(help, chunks[2]);
}

/// Build the list of context-sensitive key hints to show at the bottom of the
/// menu panel for the current page/focus. Returns `(label, key)` pairs.
fn context_hints(app: &App) -> Vec<(&'static str, &'static str)> {
    match app.state.selected_menu {
        MenuFocus::Games => {
            let mut hints = Vec::new();
            // Play-by-play toggle is available on all game views except pre-game.
            if app.state.games.focus != GamesFocus::Pregame {
                hints.push(("Play-by-play", "p"));
            }
            // When the play-by-play pane is open, Tab switches focus between it
            // and the info pane.
            if app.state.games.plays_visible && app.state.games.focus != GamesFocus::Pregame {
                hints.push(("Toggle focus", "tab"));
            }
            if app.state.games.focus == GamesFocus::Boxscore {
                hints.extend([
                    ("Forwards", "f"),
                    ("Defense", "d"),
                    ("Goalies", "g"),
                    ("Switch team", "t"),
                ]);
            }
            hints
        }
        _ => vec![],
    }
}

fn render_date_picker(f: &mut Frame, app: &mut App, rect: Rect) {
    let chunk = LayoutAreas::create_picker_rect(rect);
    match app.state.selected_menu {
        MenuFocus::Games | MenuFocus::Standings => {
            f.render_stateful_widget(DateSelectorWidget {}, chunk, &mut app.state.date_state)
        }
        MenuFocus::Playoffs | MenuFocus::TeamStats => {
            f.render_stateful_widget(YearSelectorWidget {}, chunk, &mut app.state.date_state)
        }
    }
    let (cx, cy) = popup_cursor_position(chunk, app.state.date_state.text.len() as u16);
    f.set_cursor_position((cx, cy));
}

fn render_team_picker(f: &mut Frame, app: &mut App, rect: Rect) {
    let chunk = LayoutAreas::create_picker_rect(rect);
    f.render_stateful_widget(
        TeamSelectorWidget {},
        chunk,
        &mut app.state.team_stats.team_picker,
    );

    let (cx, cy) = popup_cursor_position(chunk, app.state.team_stats.team_picker.text.len() as u16);
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
        .border_style(border_style());
    frame.render_widget(block, area);

    frame.render_stateful_widget(HelpWidget {}, area, &mut app.state.help.table_state);
}
