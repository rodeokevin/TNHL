use crate::models::game_story::{GameStoryReponse, ShootoutAttemptResult};
use crate::models::games::{
    GameData, GameState, GoalModifier, GoalStrength, PeriodDescriptor, PeriodType,
};
use crate::ui::render::BORDER_FOCUSED_COLOR;
use std::rc::Rc;
use std::vec;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

pub fn render_scoring(
    game: &GameData,
    maybe_game_story: Option<&GameStoryReponse>,
    frame: &mut Frame,
    area: Rect,
    scroll_offset: usize,
    max_scoring_scroll: &mut usize,
) {
    let away_team_abbrev = &game.away_team.abbrev;
    let home_team_abbrev = &game.home_team.abbrev;

    let mut away_lines = vec![];
    let mut middle_lines = vec![];
    let mut home_lines = vec![];

    // Add scoring lines
    if let Some(goals) = &game.goals
        && !goals.is_empty()
    {
        away_lines.push(Line::from("Scoring").style(Style::default().fg(BORDER_FOCUSED_COLOR)));
        home_lines.push(Line::from(""));
        let mut current_period = 0;
        for goal in goals {
            if matches!(goal.period_descriptor.period_type, PeriodType::SO) {
                continue;
            }
            // Period
            if goal.period_descriptor.number > current_period {
                if middle_lines.len() != 0 {
                    away_lines.push(Line::from("")); // keep rows synced
                    home_lines.push(Line::from(""));
                }
                current_period = goal.period_descriptor.number;
                middle_lines.push(
                    Line::from(get_period_title(&goal.period_descriptor))
                        .alignment(Alignment::Center)
                        .style(Style::default().fg(BORDER_FOCUSED_COLOR)),
                );
            }
            let goals_to_date = goal
                .goals_to_date
                .map(|n| format!(" ({})", n))
                .unwrap_or_default();
            let mut strengths = vec![];
            match goal.strength {
                GoalStrength::PP => strengths.push("PPG"),
                GoalStrength::SH => strengths.push("SHG"),
                GoalStrength::EV => {}
                _ => {}
            }
            match goal.goal_modifier {
                GoalModifier::EmptyNet => strengths.push("ENG"),
                GoalModifier::PenaltyShot => strengths.push("Penalty shot"),
                _ => {}
            }

            if goal.team_abbrev == *away_team_abbrev {
                let mut away_spans = vec![];

                if !strengths.is_empty() {
                    let label = strengths.join(", ");
                    away_spans.push(Span::styled(
                        format!("[{}] ", label),
                        Style::default().fg(Color::Red),
                    ));
                }

                away_spans.push(Span::raw(format!(
                    "{} {}{}",
                    goal.first_name, goal.last_name, goals_to_date
                )));

                away_lines.push(Line::from(away_spans).alignment(Alignment::Right));
                home_lines.push(Line::from(""));
                middle_lines.push(Line::from(""));

                if !goal.assists.is_empty() {
                    let assists_text = goal
                        .assists
                        .iter()
                        .map(|assist| {
                            format!(
                                "{} ({})",
                                assist.name,
                                assist.assists_to_date.unwrap_or_default()
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ");

                    away_lines.push(
                        Line::styled(
                            format!("[{}]", assists_text),
                            Style::default().fg(Color::DarkGray),
                        )
                        .alignment(Alignment::Right),
                    );
                } else {
                    away_lines.push(
                        Line::styled("[Unassisted]", Style::default().fg(Color::DarkGray))
                            .alignment(Alignment::Right),
                    );
                }
                home_lines.push(Line::from(""));
                middle_lines.push(Line::from(""));

                home_lines.push(Line::from(""));
                away_lines.push(Line::from(""));
                middle_lines.push(Line::from(""));
            } else if goal.team_abbrev == *home_team_abbrev {
                let mut home_spans = vec![Span::raw(format!(
                    "{} {}{}",
                    goal.first_name, goal.last_name, goals_to_date
                ))];

                if !strengths.is_empty() {
                    let label = strengths.join(", ");
                    home_spans.push(Span::styled(
                        format!(" [{}]", label),
                        Style::default().fg(Color::Red),
                    ));
                }

                away_lines.push(Line::from(""));
                home_lines.push(Line::from(home_spans));
                middle_lines.push(Line::from(""));

                if !goal.assists.is_empty() {
                    let assists_text = goal
                        .assists
                        .iter()
                        .map(|assist| {
                            format!(
                                "{} ({})",
                                assist.name,
                                assist.assists_to_date.unwrap_or_default()
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ");

                    home_lines.push(
                        Line::styled(
                            format!("[{}]", assists_text),
                            Style::default().fg(Color::DarkGray),
                        )
                        .alignment(Alignment::Left),
                    );
                } else {
                    home_lines.push(
                        Line::styled("[Unassisted]", Style::default().fg(Color::DarkGray))
                            .alignment(Alignment::Left),
                    );
                }
                away_lines.push(Line::from(""));
                middle_lines.push(Line::from(""));

                home_lines.push(Line::from(""));
                away_lines.push(Line::from(""));
                middle_lines.push(Line::from(""));
            }
        }
        // Add shootout attempts if there are any
        if let Some(game_story) = maybe_game_story
            && let Some(summary) = &game_story.summary
            && !summary.shootout.is_empty()
        {
            // Add shootout lines
            away_lines.push(Line::from(""));
            middle_lines.push(
                Line::from("Shootout")
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(BORDER_FOCUSED_COLOR)),
            );
            home_lines.push(Line::from(""));
            for shootout_attempt in &summary.shootout {
                let (attempt_symbol, attempt_color) =
                    if matches!(shootout_attempt.result, ShootoutAttemptResult::Goal) {
                        ("[✓]", Color::Green)
                    } else {
                        ("[✗]", Color::Red)
                    };

                let attempt_span = Span::styled(attempt_symbol, Style::default().fg(attempt_color));

                if shootout_attempt.team_abbrev.default == *away_team_abbrev {
                    away_lines.push(
                        Line::from(vec![
                            Span::raw(format!(
                                "{} {} ",
                                shootout_attempt.first_name, shootout_attempt.last_name
                            )),
                            attempt_span,
                        ])
                        .alignment(Alignment::Right),
                    );
                    middle_lines.push(Line::from(""));
                    home_lines.push(Line::from(""));
                } else {
                    home_lines.push(
                        Line::from(vec![
                            attempt_span,
                            Span::raw(format!(
                                " {} {}",
                                shootout_attempt.first_name, shootout_attempt.last_name
                            )),
                        ])
                        .alignment(Alignment::Left),
                    );
                    middle_lines.push(Line::from(""));
                    away_lines.push(Line::from(""));
                }
            }
        }
    } else if matches!(game.game_state, GameState::LIVE | GameState::CRIT) {
        // No goals yet but game is live (but not shootout)
        away_lines.push(Line::from("Scoring").style(Style::default().fg(Color::DarkGray)));
        middle_lines.push(
            Line::from("\"No goals.\" - Juuse Saros").style(Style::default().fg(Color::DarkGray)),
        );
        home_lines.push(Line::from(""));
    }

    // Split area into top scroll indicator, content and bottom scroll indicator
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
    let visible_period: Vec<Line> = middle_lines[offset..end].iter().cloned().collect();

    frame.render_widget(
        Line::from(if can_scroll_up { "▲" } else { "" }).alignment(Alignment::Center),
        vert_chunks[0],
    );
    frame.render_widget(
        Line::from(if can_scroll_down { "▼" } else { "" }).alignment(Alignment::Center),
        vert_chunks[2],
    );

    // Re-split the content area horizontally
    let chunks = split_info_left_middle_right(vert_chunks[1], 25);

    frame.render_widget(Paragraph::new(visible_away), chunks[0]);
    frame.render_widget(Paragraph::new(visible_period), chunks[1]);
    frame.render_widget(Paragraph::new(visible_home), chunks[2]);
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

// Helper to create the areas for left-center-right
fn split_info_left_middle_right(area: Rect, middle_length: u16) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(middle_length),
            Constraint::Fill(1),
        ])
        .split(area)
}
