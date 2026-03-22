use crate::app::App;
use crate::models::games::{GameData, GameState, PeriodType, SituationDesc};
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
            .fg(Color::Rgb(247, 194, 0))
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
            let upper_info_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // Time remaining
                    Constraint::Length(1), // Status bar
                    Constraint::Length(1), // Teams and strength status
                    Constraint::Min(0),
                ])
                .split(upper_score_lower[0]);
            render_time_remaining(game, frame, upper_info_chunks[0]); // Time remaining
            render_sweeping_status(game, 10, app.sweeping_status_offet, frame, upper_info_chunks[1]); // Status bar
            render_team_status(game, frame, upper_info_chunks[2]); // Teams and strength status

            // Score
            let score = BigText::builder()
                .pixel_size(PixelSize::Sextant)
                .style(Style::default().fg(Color::Green))
                .lines(vec![
                    format!("{} - {}", away.score.unwrap_or(0), home.score.unwrap_or(0)).into(),
                ])
                .centered()
                .build();
            frame.render_widget(score, upper_score_lower[1]);
            // Lower info
            let mut lower_info_lines = vec![
                Line::from("Scoring").style(
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
            ];
            let lower_info_paragraph =
                Paragraph::new(lower_info_lines).style(Style::default().fg(Color::White));

            frame.render_widget(lower_info_paragraph, upper_score_lower[2]);
        }
    } else {
        // Todo: no data
    }
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

pub fn render_time_remaining(game: &GameData, frame: &mut Frame, area: Rect) {
    match game.game_state {
        GameState::FUT | GameState::PRE => {
            frame.render_widget(
                Line::from(format!("{}", game.start_time_utc)).alignment(Alignment::Center),
                area,
            );
        }
        GameState::LIVE | GameState::CRIT => {
            if let Some(clock) = &game.clock {
                if clock.in_intermission {
                    match game.period_descriptor.as_ref().unwrap().period_type {
                        PeriodType::REG => frame.render_widget(
                            Line::from(format!(
                                "End of Period {} ({})",
                                game.period.unwrap_or(0),
                                game.clock.as_ref().unwrap().time_remaining
                            ))
                            .alignment(Alignment::Center),
                            area,
                        ),
                        PeriodType::OT => frame.render_widget(
                            Line::from(format!(
                                "End of OT{}",
                                game.period_descriptor
                                    .as_ref()
                                    .unwrap()
                                    .ot_periods
                                    .unwrap_or(0)
                            ))
                            .alignment(Alignment::Center),
                            area,
                        ),
                        PeriodType::SO => frame.render_widget(
                            Line::from("End of Shootout").alignment(Alignment::Center),
                            area,
                        ),
                        _ => frame.render_widget(
                            Line::from("Intermission").alignment(Alignment::Center),
                            area,
                        ),
                    }
                } else {
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Fill(1),
                            Constraint::Length(10), // middle part
                            Constraint::Fill(1),
                        ])
                        .split(area);
                    let time = Line::from(format!(
                        "P{} - {}",
                        game.period.unwrap_or(0),
                        clock.time_remaining,
                    )).alignment(Alignment::Right);
                    frame.render_widget(time, chunks[1]);

                    let mut spans = vec![];
                    if let Some(s) = &game.situation {
                        if s.away_team.strength > s.home_team.strength {
                            spans.push(Span::styled(
                                format!(" {}-on-{}", s.away_team.strength, s.home_team.strength),
                                Style::default().fg(Color::Red),
                            ));
                        } else {
                            spans.push(Span::styled(
                                format!(" {}-on-{}", s.home_team.strength, s.away_team.strength),
                                Style::default().fg(Color::Red),
                            ))
                        }
                    }
                    frame.render_widget(Line::from(spans).alignment(Alignment::Left), chunks[2]);
                }
            } else {
                frame.render_widget(Line::from("Live").alignment(Alignment::Center), area);
            }
        }
        GameState::OVER | GameState::FINAL | GameState::OFF => {
            match game.game_outcome.as_ref().unwrap().last_period_type {
                PeriodType::REG | PeriodType::Unknown => frame.render_widget(
                    Line::from(format!("Final")).alignment(Alignment::Center),
                    area,
                ),
                PeriodType::OT => {
                    if game.game_outcome.as_ref().unwrap().ot_periods.unwrap_or(0) > 1 {
                        frame.render_widget(
                            Line::from(format!(
                                "Final/{}OT",
                                game.game_outcome.as_ref().unwrap().ot_periods.unwrap()
                            ))
                            .alignment(Alignment::Center),
                            area,
                        );
                    } else {
                        frame.render_widget(
                            Line::from(format!("Final/OT")).alignment(Alignment::Center),
                            area,
                        );
                    }
                }
                PeriodType::SO => frame.render_widget(
                    Line::from(format!("Final/SO")).alignment(Alignment::Center),
                    area,
                ),
            }
        }
        GameState::Unknown => frame.render_widget(
            Line::from("Unknown game state").alignment(Alignment::Center),
            area,
        ),
    }
}

pub fn render_sweeping_status(
    game: &GameData,
    width: usize,
    offset: usize,
    frame: &mut Frame,
    area: Rect,
) {
    match game.game_state {
        GameState::FUT
        | GameState::PRE
        | GameState::OVER
        | GameState::FINAL
        | GameState::OFF
        | GameState::Unknown => frame.render_widget(Line::from(""), area),
        GameState::LIVE | GameState::CRIT => {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Fill(1),
                    Constraint::Length(10), // middle part to align with the time above
                    Constraint::Fill(1),
                ])
                .split(area);
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
                    frame.render_widget(Line::from(spans).alignment(Alignment::Center), chunks[1]);
                } else {
                    let spans: Vec<_> = (0..width)
                        .map(|_i| Span::styled("─", Style::default().fg(Color::Red)))
                        .collect();

                    frame.render_widget(Line::from(spans).alignment(Alignment::Center), chunks[1]);
                }
            } else {
                frame.render_widget(Line::from(""), area);
            }
        }
    }
}

pub fn render_team_status(game: &GameData, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(6), // middle
            Constraint::Fill(1),
        ])
        .split(area);

    let mut left_spans = vec![];
    let situation = game.situation.as_ref();

    if let Some(s) = situation {
        if let Some(descs) = s.away_team.situation_descriptions.as_deref() {
            let parts: Vec<String> = descs
                .iter()
                .map(|d| match d {
                    SituationDesc::PP => {
                        format!("PP: {}", s.time_remaining)
                    }
                    SituationDesc::EN => "EN".to_string(),
                    SituationDesc::Unknown => "Unknown".to_string(),
                })
                .collect();

            if !parts.is_empty() {
                let label = format!("[{}] ", parts.join(", "));
                left_spans.push(Span::styled(label, Style::default().fg(Color::Red)));
            }
        }
    }
    left_spans.push(Span::raw(&game.away_team.name.default));
    frame.render_widget(Line::from(left_spans).alignment(Alignment::Right), chunks[0]);
    frame.render_widget(Line::from("  vs  ").alignment(Alignment::Center), chunks[1]);

    let mut right_spans = vec![];
    right_spans.push(Span::raw(&game.home_team.name.default));
    if let Some(s) = situation {
        if let Some(descs) = s.home_team.situation_descriptions.as_deref() {
            let parts: Vec<String> = descs
                .iter()
                .map(|d| match d {
                    SituationDesc::PP => {
                        format!("PP: {}", s.time_remaining)
                    }
                    SituationDesc::EN => "EN".to_string(),
                    SituationDesc::Unknown => "Unknown".to_string(),
                })
                .collect();
            if !parts.is_empty() {
                let label = format!(" [{}] ", parts.join(", "));
                right_spans.push(Span::styled(label, Style::default().fg(Color::Red)));
            }
        }
    }
    frame.render_widget(Line::from(right_spans).alignment(Alignment::Left), chunks[2]);
}
