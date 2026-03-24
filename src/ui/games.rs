use crate::app::App;
use crate::models::games::{
    GameData, GameState, GoalStrength, PeriodDescriptor, PeriodType, SituationDesc,
};
use crate::ui::PaneFocus;
use chrono_tz::Tz;
use core::time;
use std::vec;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Tabs},
};

use tui_big_text::{BigText, PixelSize};

pub fn render_games(frame: &mut Frame, app: &mut App, area: Rect) {
    // Split content chunk into tab + content
    let tab_content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // tabs
            Constraint::Min(1),    // table
        ])
        .split(area);

    let matchups: Vec<Line> = app
        .state
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

    let focused = app.state.focus == PaneFocus::Content;
    let border_style = if focused {
        Style::default()
            .fg(Color::Rgb(247, 194, 0))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let selected_color = Style::default()
        .add_modifier(Modifier::BOLD)
        .add_modifier(Modifier::UNDERLINED);

    let tabs = Tabs::new(matchups)
        .select(app.state.selected_game_index)
        .block(
            Block::bordered()
                .border_style(border_style)
                .title(app.state.date_selector.format_date_border_title()),
        )
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
            Constraint::Length(5), // upper info
            Constraint::Length(4), // score (big text)
            Constraint::Fill(1),   // lower info
        ])
        .split(inner);

    // Render game information
    if let Some(games_data) = &mut app.state.games_data {
        if let Some(game) = games_data.games.get(app.state.selected_game_index) {
            // Upper info
            let upper_info_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // Time remaining
                    Constraint::Length(1), // Status bar
                    Constraint::Length(1), // Teams and strength status
                    Constraint::Length(1), // Shots on goal
                ])
                .split(upper_score_lower[0]);
            render_time_remaining(game, app.settings.timezone, frame, upper_info_chunks[0]);
            render_sweeping_status(
                game,
                10,
                app.state.sweeping_status_offset,
                frame,
                upper_info_chunks[1],
            );
            render_team_status(game, frame, upper_info_chunks[2]);
            render_shots_on_goal(game, frame, upper_info_chunks[3]);

            // Score in big text
            render_big_score(game, frame, upper_score_lower[1]);

            // Lower info
            let lower_info_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0), // Scoring
                ])
                .split(upper_score_lower[2]);
            render_scoring(
                game,
                frame,
                lower_info_chunks[0],
                app.state.scoring_scroll_offset,
                &mut app.state.max_scoring_scroll,
            );
        }
    } else {
        // Todo: no data
    }
}

pub fn get_color_from_game_state(state: &GameState) -> Style {
    match state {
        GameState::LIVE | GameState::CRIT | GameState::OVER => Style::default().fg(Color::Green),
        GameState::FINAL | GameState::OFF => Style::default().fg(Color::DarkGray),
        _ => Style::default().fg(Color::White), // FUT, PRE, Unknown
    }
}

pub fn render_time_remaining(game: &GameData, timezone: Tz, frame: &mut Frame, area: Rect) {
    // Not in intermission
    if matches!(game.game_state, GameState::LIVE | GameState::CRIT) {
        if let Some(clock) = &game.clock {
            if !clock.in_intermission {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Fill(1),
                        Constraint::Length(22),
                        Constraint::Fill(1),
                    ])
                    .split(area);

                let time = Line::from(format!(
                    "{} - {}",
                    get_period_title(game.period_descriptor.as_ref().unwrap()),
                    clock.time_remaining,
                ))
                .alignment(Alignment::Center);
                frame.render_widget(time, chunks[1]);

                let spans: Vec<Span> = game
                    .situation
                    .as_ref()
                    .map(|s| {
                        vec![Span::styled(
                            format!(
                                " {}-on-{}",
                                s.away_team.strength.max(s.home_team.strength),
                                s.away_team.strength.min(s.home_team.strength)
                            ),
                            Style::default().fg(Color::Red),
                        )]
                    })
                    .unwrap_or_default();
                frame.render_widget(Line::from(spans).alignment(Alignment::Left), chunks[2]);
                return;
            }
        }
    }
    let line = match game.game_state {
        GameState::FUT | GameState::PRE => Line::from(format!(
            "{}",
            game.compute_local_time(timezone)
                .format("%-I:%M %p")
                .to_string()
        )),
        GameState::LIVE | GameState::CRIT => {
            // in intermission or clock is None
            match game.clock.as_ref() {
                None => Line::from("Live"),
                Some(_) => match game.period_descriptor.as_ref().unwrap().period_type {
                    PeriodType::REG | PeriodType::OT => Line::from(format!(
                        "End of {} ({})",
                        get_period_title(game.period_descriptor.as_ref().unwrap()),
                        game.clock.as_ref().unwrap().time_remaining
                    )),
                    PeriodType::SO => Line::from("End of Shootout"),
                    _ => Line::from("Intermission"),
                },
            }
        }
        GameState::OVER | GameState::FINAL | GameState::OFF => {
            let outcome = game.game_outcome.as_ref().unwrap();
            match outcome.last_period_type {
                PeriodType::REG | PeriodType::Unknown => Line::from("Final"),
                PeriodType::OT => match outcome.ot_periods.unwrap_or(0) {
                    n if n > 1 => Line::from(format!("Final/{}OT", n)),
                    _ => Line::from("Final/OT"),
                },
                PeriodType::SO => Line::from("Final/SO"),
            }
        }
        GameState::Unknown => Line::from("Unknown game state"),
    };

    frame.render_widget(line.alignment(Alignment::Center), area);
}

pub fn render_sweeping_status(
    game: &GameData,
    width: usize,
    offset: usize,
    frame: &mut Frame,
    area: Rect,
) {
    match game.game_state {
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
                    let spans: Vec<_> =
                        std::iter::repeat(Span::styled("─", Style::default().fg(Color::Red)))
                            .take(width)
                            .collect();

                    frame.render_widget(Line::from(spans).alignment(Alignment::Center), chunks[1]);
                }
            } else {
                frame.render_widget(Line::from(""), area);
            }
        }
        _ => frame.render_widget(Line::from(""), area),
    }
}

pub fn render_team_status(game: &GameData, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),    // Strength status if necessary
            Constraint::Length(10), // middle
            Constraint::Fill(1),    // Strength status if necessary
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
    frame.render_widget(
        Line::from(left_spans).alignment(Alignment::Right),
        chunks[0],
    );
    frame.render_widget(
        Line::from("    vs    ").alignment(Alignment::Center),
        chunks[1],
    );

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
    frame.render_widget(
        Line::from(right_spans).alignment(Alignment::Left),
        chunks[2],
    );
}

pub fn render_big_score(game: &GameData, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(10), // space for the "-"
            Constraint::Fill(1),
        ])
        .split(area);

    let away_score = BigText::builder()
        .pixel_size(PixelSize::Sextant)
        .style(Style::default().fg(Color::Green))
        .lines(vec![
            format!("{}", game.away_team.score.unwrap_or(0)).into(),
        ])
        .right_aligned()
        .build();
    frame.render_widget(away_score, chunks[0]);

    let dash = BigText::builder()
        .pixel_size(PixelSize::Sextant)
        .style(Style::default().fg(Color::Green))
        .lines(vec!["-".into()])
        .centered()
        .build();
    frame.render_widget(dash, chunks[1]);

    let home_score = BigText::builder()
        .pixel_size(PixelSize::Sextant)
        .style(Style::default().fg(Color::Green))
        .lines(vec![
            format!("{}", game.home_team.score.unwrap_or(0)).into(),
        ])
        .left_aligned()
        .build();
    frame.render_widget(home_score, chunks[2]);
}

pub fn render_shots_on_goal(game: &GameData, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(10),
            Constraint::Fill(1),
        ])
        .split(area);
    frame.render_widget(
        Line::from(format!("SOG: {}", game.away_team.sog.unwrap_or(0)))
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Right),
        chunks[0],
    );
    frame.render_widget(
        Line::from(format!("SOG: {}", game.home_team.sog.unwrap_or(0)))
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Left),
        chunks[2],
    );
}

pub fn render_scoring(
    game: &GameData,
    frame: &mut Frame,
    area: Rect,
    scroll_offset: usize,
    max_scoring_scroll: &mut usize,
) {
    let away_team_abbrev = &game.away_team.abbrev;
    let home_team_abbrev = &game.home_team.abbrev;

    if let Some(goals) = &game.goals {
        let mut away_lines = vec![Line::from("Scoring").style(Style::default().fg(Color::Blue))];
        let mut home_lines = vec![Line::from("")];
        let mut period_lines = vec![];
        let mut current_period = 0;
        for goal in goals {
            // Period
            if goal.period_descriptor.number > current_period {
                if current_period != 0 {
                    away_lines.push(Line::from("")); // keep rows synced
                    home_lines.push(Line::from("")); // keep rows synced
                }
                current_period = goal.period_descriptor.number;
                period_lines.push(
                    Line::from(get_period_title(&goal.period_descriptor))
                        .alignment(Alignment::Center)
                        .style(Style::default().fg(Color::Blue)),
                );
            }
            let goals_to_date = goal
                .goals_to_date
                .map(|n| format!(" ({})", n))
                .unwrap_or_default();
            let strength = match goal.strength {
                GoalStrength::PP => Some("PPG"),
                GoalStrength::EmptyNet => Some("ENG"),
                GoalStrength::SH => Some("SHG"),
                _ => None,
            };

            if goal.team_abbrev == *away_team_abbrev {
                let mut away_spans = vec![];
                if let Some(s) = strength {
                    away_spans.push(Span::styled(
                        format!("[{}] ", s),
                        Style::default().fg(Color::Red),
                    ));
                }
                away_spans.push(Span::raw(format!(
                    "{} {}{}",
                    goal.first_name, goal.last_name, goals_to_date
                )));

                away_lines.push(Line::from(away_spans).alignment(Alignment::Right));
                home_lines.push(Line::from("")); // keep rows synced
                period_lines.push(Line::from(""));
            } else if goal.team_abbrev == *home_team_abbrev {
                let mut home_spans = vec![Span::raw(format!(
                    "{} {}{}",
                    goal.first_name, goal.last_name, goals_to_date
                ))];
                if let Some(s) = strength {
                    home_spans.push(Span::styled(
                        format!(" [{}]", s),
                        Style::default().fg(Color::Red),
                    ));
                }
                away_lines.push(Line::from(""));
                home_lines.push(Line::from(home_spans).alignment(Alignment::Left));
                period_lines.push(Line::from(""));
            }
        }

        // Split area into top indicator, content, bottom indicator
        let vert_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(area);

        let content_height = vert_chunks[1].height as usize;
        let max_scroll = away_lines.len().saturating_sub(content_height);
        *max_scoring_scroll = max_scroll;
        let offset = scroll_offset.min(max_scroll);
        let can_scroll_up = offset > 0;
        let can_scroll_down = offset < max_scroll;

        // Slice to visible window
        let end = (offset + content_height).min(away_lines.len());

        let visible_away: Vec<Line> = away_lines[offset..end].iter().cloned().collect();
        let visible_home: Vec<Line> = home_lines[offset..end].iter().cloned().collect();
        let visible_period: Vec<Line> = period_lines[offset..end].iter().cloned().collect();

        frame.render_widget(
            Line::from(if can_scroll_up { "▲" } else { "" }).alignment(Alignment::Center),
            vert_chunks[0],
        );
        frame.render_widget(
            Line::from(if can_scroll_down { "▼" } else { "" }).alignment(Alignment::Center),
            vert_chunks[2],
        );

        // Re-split the content area horizontally
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(14),
                Constraint::Fill(1),
            ])
            .split(vert_chunks[1]);

        frame.render_widget(Paragraph::new(visible_away), chunks[0]);
        frame.render_widget(Paragraph::new(visible_period), chunks[1]);
        frame.render_widget(Paragraph::new(visible_home), chunks[2]);
    } else if matches!(game.game_state, GameState::LIVE | GameState::CRIT) {
        // No goals yet but game is live
        frame.render_widget(
            Paragraph::new("Scoring")
                .alignment(Alignment::Left)
                .style(Style::default().fg(Color::Blue)),
            area,
        );
        frame.render_widget(
            Paragraph::new("\"No goals.\" - Juuse Saros")
                .alignment(Alignment::Left)
                .style(Style::default().fg(Color::DarkGray)),
            area,
        );
    }
}

pub fn get_period_title(period: &PeriodDescriptor) -> String {
    match period.period_type {
        PeriodType::REG => match period.number {
            1 => "1st Period".to_string(),
            2 => "2nd Period".to_string(),
            3 => "3rd Period".to_string(),
            _ => format!("{}th Period", period.number).to_string(),
        },
        PeriodType::OT => match period.ot_periods.unwrap_or(0) {
            0 | 1 => "Overtime".to_string(),
            2 => "2nd Overtime".to_string(),
            3 => "3rd Overtime".to_string(),
            _ => format!("{}th Overtime", period.ot_periods.unwrap_or(0)),
        },
        PeriodType::SO => "Shootout".to_string(),
        _ => "Unknown Period".to_string(),
    }
}
