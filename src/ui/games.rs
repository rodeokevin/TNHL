use crate::app::App;
use crate::models::games::{GameData, GameState, PeriodType};
use crate::ui::PaneFocus;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Tabs},
};

use tui_big_text::{BigText, PixelSize};

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
                    let color = get_color_from_game_state(&game.game_state);
                    Line::from(format!(
                        "{} @ {}",
                        game.away_team.abbrev, game.home_team.abbrev
                    ))
                    .style(color)
                })
                .collect()
        })
        .unwrap_or_default();

    let focused = app.focus == PaneFocus::Content;
    let border_style = if focused {
        Style::default()
            .fg(Color::LightYellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let selected_color = if focused {
        Style::default()
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::UNDERLINED)
    } else {
        Style::default()
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::UNDERLINED)
    };

    let tabs = Tabs::new(titles)
        .select(app.selected_game_index)
        .block(Block::bordered().border_style(border_style))
        .highlight_style(selected_color);

    frame.render_widget(tabs, tab_content_chunks[0]);

    let block = Block::bordered()
        .title(" Overview ")
        .border_style(border_style);
    frame.render_widget(block.clone(), tab_content_chunks[1]);

    let inner = block.inner(tab_content_chunks[1]);

    let upper_score_lower = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),       // upper info
            Constraint::Length(4),       // score (bigger)
            Constraint::Percentage(100), // lower info
        ])
        .split(inner);

    // Render game information
    if let Some(games_data) = &app.games_data {
        if let Some(game) = games_data.games.get(app.selected_game_index) {
            let home = &game.home_team;
            let away = &game.away_team;

            // Upper info
            let mut lines = vec![time_remaining(game)]; // Time remaining
            let sweeping_status = sweeping_status(game, 10, app.sweeping_status_offet); // Status bar
            lines.push(sweeping_status);
            let teams = Line::from(format!(
                "{}   vs   {}",
                away.name.default, home.name.default
            ))
            .alignment(Alignment::Center); // Teams
            lines.push(teams);

            let paragraph = Paragraph::new(lines).style(Style::default().fg(Color::White));

            frame.render_widget(paragraph, upper_score_lower[0]);

            // Score
            let score = BigText::builder()
                .pixel_size(PixelSize::Sextant)
                .style(Style::default().fg(Color::Green))
                .lines(vec![
                    format!("{} - {}", away.score.unwrap_or(0), home.score.unwrap_or(0)).into(),
                ])
                .centered()
                .build();
            frame.render_widget(score, upper_score_lower[1])
        }
    }

    // Lower info
}

pub fn get_color_from_game_state(state: &GameState) -> Style {
    match state {
        GameState::FUT => Style::default().fg(Color::White),
        GameState::PRE => Style::default().fg(Color::White),
        GameState::LIVE => Style::default().fg(Color::Green),
        GameState::CRIT => Style::default().fg(Color::Green),
        GameState::OVER => Style::default().fg(Color::Green),
        GameState::FINAL => Style::default().fg(Color::DarkGray),
        GameState::OFF => Style::default().fg(Color::DarkGray),
        GameState::Unknown => Style::default().fg(Color::White),
    }
}

pub fn time_remaining(game: &GameData) -> Line<'_> {
    match game.game_state {
        GameState::FUT | GameState::PRE => Line::from(format!("{}", game.start_time_utc)),
        GameState::LIVE | GameState::CRIT => {
            if let Some(clock) = &game.clock {
                if clock.in_intermission {
                    let period = game.period.unwrap_or(0);
                    Line::from(format!("End of Period {}", period)).alignment(Alignment::Center)
                } else {
                    Line::from(format!(
                        "P{} - {}",
                        game.period.unwrap_or(0),
                        clock.time_remaining
                    ))
                    .alignment(Alignment::Center)
                }
            } else {
                Line::from("Live").alignment(Alignment::Center)
            }
        }
        GameState::OVER | GameState::FINAL | GameState::OFF => {
            match game.game_outcome.as_ref().unwrap().last_period_type {
                PeriodType::REG | PeriodType::Unknown => {
                    Line::from(format!("Final")).alignment(Alignment::Center)
                }
                PeriodType::OT => {
                    if game.game_outcome.as_ref().unwrap().ot_periods.unwrap_or(0) > 1 {
                        Line::from(format!(
                            "Final/{}OT",
                            game.game_outcome.as_ref().unwrap().ot_periods.unwrap()
                        ))
                        .alignment(Alignment::Center)
                    } else {
                        Line::from(format!("Final/OT")).alignment(Alignment::Center)
                    }
                }
                PeriodType::SO => Line::from(format!("Final/SO")).alignment(Alignment::Center),
            }
        }
        GameState::Unknown => Line::from("Unknown game state").alignment(Alignment::Center),
    }
}

pub fn sweeping_status(game: &GameData, width: usize, offset: usize) -> Line<'_> {
    match game.game_state {
        GameState::FUT
        | GameState::PRE
        | GameState::OVER
        | GameState::FINAL
        | GameState::OFF
        | GameState::Unknown => Line::from(""),
        GameState::LIVE | GameState::CRIT => {
            if let Some(clock) = &game.clock {
                if clock.running {
                    let spans: Vec<_> = (0..width)
                        .map(|i| {
                            let pos = offset % width;
                            let dist = i.abs_diff(pos).min(width - i.abs_diff(pos));
                            if dist == 0 || dist == 1 {
                                Span::styled("━", Style::default().fg(Color::Green))
                            } else {
                                Span::styled("─", Style::default().fg(Color::DarkGray))
                            }
                        })
                        .collect();
                    Line::from(spans).alignment(Alignment::Center)
                } else {
                    let spans: Vec<_> = (0..width)
                        .map(|_i| Span::styled("─", Style::default().fg(Color::Red)))
                        .collect();

                    Line::from(spans).alignment(Alignment::Center)
                }
            } else {
                Line::from("")
            }
        }
    }
}

// Team name
// pub fn teams(game: &GameData) {
//     let home = &game.home_team;
//     let away = &game.away_team;

//     let big_text = BigText::builder()
//         .pixel_size(PixelSize::Full)
//         .style(Style::new().blue())
//         .lines(vec![
//             "Hello".into(),
//             "World".into(),
//             "~~~~~".into(),
//         ])
//         .centered()
//         .build();
//     frame.render_widget(big_text, area);
// }
