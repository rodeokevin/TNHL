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
use serde_with::NoneAsEmptyString;

pub fn render_scoring_and_stats(
    game: &GameData,
    frame: &mut Frame,
    area: Rect,
    scroll_offset: usize,
    max_scoring_scroll: &mut usize,
) {
    let away_team_abbrev = &game.away_team.abbrev;
    let home_team_abbrev = &game.home_team.abbrev;

    if let Some(goals) = &game.goals
        && !goals.is_empty()
    {
        let mut away_lines =
            vec![Line::from("Scoring").style(Style::default().fg(BORDER_FOCUSED_COLOR))];
        let mut home_lines = vec![Line::from("")];
        let mut period_lines = vec![];
        let mut current_period = 0;
        for goal in goals {
            // Period
            if goal.period_descriptor.number > current_period {
                if current_period != 0 {
                    away_lines.push(Line::from("")); // keep rows synced
                    home_lines.push(Line::from(""));
                }
                current_period = goal.period_descriptor.number;
                period_lines.push(
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
                period_lines.push(Line::from(""));
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
                period_lines.push(Line::from(""));
            }
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
        let chunks = split_info_left_middle_right(vert_chunks[1], 25);

        frame.render_widget(Paragraph::new(visible_away), chunks[0]);
        frame.render_widget(Paragraph::new(visible_period), chunks[1]);
        frame.render_widget(Paragraph::new(visible_home), chunks[2]);
    } else if matches!(game.game_state, GameState::LIVE | GameState::CRIT) {
        // No goals yet but game is live
        frame.render_widget(
            Paragraph::new("\nScoring")
                .alignment(Alignment::Left)
                .style(Style::default().fg(BORDER_FOCUSED_COLOR)),
            area,
        );
        frame.render_widget(
            Paragraph::new("\n\"No goals.\" - Juuse Saros")
                .alignment(Alignment::Center)
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
