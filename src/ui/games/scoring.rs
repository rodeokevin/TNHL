use crate::models::games::{
    game_story::{GameStoryReponse, ShootoutAttemptResult},
    games::{
        AssistInfo, GameData, GameState, GoalModifier, GoalStrength, PeriodDescriptor, PeriodType,
    },
};
use crate::ui::{
    games::games::{BIG_SCORE_COLOR, split_info_left_middle_right},
    games::stats::AWAY_BAR_COLOR,
    render::border_style,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

// Length of the middle chunk for scoring and stats
pub const MIDDLE_LENGTH: u16 = 25;

pub fn render_scoring(
    game: &GameData,
    maybe_game_story: Option<&GameStoryReponse>,
    frame: &mut Frame,
    area: Rect,
    scroll_offset: usize,
    max_scroll: &mut usize,
    visible_rows: &mut usize,
) {
    // Pass visible rows to game state
    *visible_rows = area.height.saturating_sub(3) as usize;

    let away_team_abbrev = &game.away_team.abbrev;
    let home_team_abbrev = &game.home_team.abbrev;

    let mut away_lines = vec![];
    let mut middle_lines = vec![];
    let mut home_lines = vec![];

    // Add scoring lines
    if let Some(goals) = &game.goals
        && !goals.is_empty()
    {
        let mut current_period = 0;
        for (i, goal) in goals.iter().enumerate() {
            if matches!(goal.period_descriptor.period_type, PeriodType::SO) {
                continue;
            }
            // Period
            if goal.period_descriptor.number > current_period {
                away_lines.push(Line::default());
                home_lines.push(Line::default());
                middle_lines.push(
                    Line::from(get_period_title(&goal.period_descriptor))
                        .alignment(Alignment::Center)
                        .style(border_style()),
                );
                current_period = goal.period_descriptor.number;
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

            if goal.team_abbrev == *away_team_abbrev.to_string() {
                let mut away_spans = vec![];

                if !strengths.is_empty() {
                    let label = strengths.join(", ");
                    away_spans.push(Span::styled(
                        format!("[{}] ", label),
                        Style::default()
                            .fg(AWAY_BAR_COLOR)
                            .add_modifier(Modifier::BOLD),
                    ));
                }

                away_spans.push(Span::raw(format!(
                    "[{}] {} {}{}",
                    goal.time_in_period, goal.first_name, goal.last_name, goals_to_date
                )));

                away_lines.push(Line::from(away_spans).alignment(Alignment::Right));
                home_lines.push(Line::default());
                middle_lines.push(Line::default());

                // Add a line for assists (or display unassisted)
                away_lines.push(get_assists_line(&goal.assists, Alignment::Right));
                home_lines.push(Line::default());
                middle_lines.push(Line::default());
            } else if goal.team_abbrev == *home_team_abbrev.to_string() {
                let mut home_spans = vec![Span::raw(format!(
                    "{} {}{} [{}]",
                    goal.first_name, goal.last_name, goals_to_date, goal.time_in_period
                ))];

                if !strengths.is_empty() {
                    let label = strengths.join(", ");
                    home_spans.push(Span::styled(
                        format!(" [{}]", label),
                        Style::default()
                            .fg(AWAY_BAR_COLOR)
                            .add_modifier(Modifier::BOLD),
                    ));
                }

                away_lines.push(Line::default());
                home_lines.push(Line::from(home_spans));
                middle_lines.push(Line::default());

                home_lines.push(get_assists_line(&goal.assists, Alignment::Left));
                away_lines.push(Line::default());
                middle_lines.push(Line::default());
            }
            // Add spacing if next goal is in the same period
            if let Some(next_goal) = goals.get(i + 1) {
                if next_goal.period_descriptor.number == goal.period_descriptor.number {
                    home_lines.push(Line::default());
                    away_lines.push(Line::default());
                    middle_lines.push(Line::default());
                }
            }
        }
        // Add shootout attempts if there are any
        if let Some(game_story) = maybe_game_story
            && let Some(summary) = &game_story.summary
            && !summary.shootout.is_empty()
        {
            // Add shootout lines
            away_lines.push(Line::default());
            middle_lines.push(
                Line::from("Shootout")
                    .alignment(Alignment::Center)
                    .style(border_style()),
            );
            home_lines.push(Line::default());
            for shootout_attempt in &summary.shootout {
                let (attempt_symbol, attempt_color) =
                    if matches!(shootout_attempt.result, ShootoutAttemptResult::Goal) {
                        ("[✓]", BIG_SCORE_COLOR)
                    } else {
                        ("[✗]", AWAY_BAR_COLOR)
                    };

                let attempt_span = Span::styled(attempt_symbol, Style::default().fg(attempt_color));

                if shootout_attempt.team_abbrev.default == *away_team_abbrev.to_string() {
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
                    middle_lines.push(Line::default());
                    home_lines.push(Line::default());
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
                    middle_lines.push(Line::default());
                    away_lines.push(Line::default());
                }
            }
        }
    } else if matches!(game.game_state, GameState::LIVE | GameState::CRIT) {
        // No goals yet but game is live (but not shootout)
        middle_lines.push(
            Line::from("\"No goals.\" - Juuse Saros").style(Style::default().fg(Color::DarkGray)),
        );
        home_lines.push(Line::default());
        away_lines.push(Line::default());
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
    let last_line = away_lines.len().saturating_sub(content_height);
    *max_scroll = last_line;
    let offset = scroll_offset.min(last_line);
    let can_scroll_up = offset > 0;
    let can_scroll_down = offset < last_line;

    // Slice to visible window
    let end = (offset + content_height).min(away_lines.len());

    let visible_away = away_lines[offset..end].to_vec();
    let visible_home = home_lines[offset..end].to_vec();
    let visible_middle = middle_lines[offset..end].to_vec();

    frame.render_widget(
        Line::from(if can_scroll_up { "▲" } else { "" }).alignment(Alignment::Center),
        vert_chunks[0],
    );
    frame.render_widget(
        Line::from(if can_scroll_down { "▼" } else { "" }).alignment(Alignment::Center),
        vert_chunks[2],
    );

    // Re-split the content area horizontally
    let chunks = split_info_left_middle_right(vert_chunks[1], MIDDLE_LENGTH);

    frame.render_widget(Paragraph::new(visible_away), chunks[0]);
    frame.render_widget(Paragraph::new(visible_middle), chunks[1]);
    frame.render_widget(Paragraph::new(visible_home), chunks[2]);
}

fn get_assists_line(assists: &Vec<AssistInfo>, alignment: Alignment) -> Line<'static> {
    if !assists.is_empty() {
        let assists_text = get_assists_text(assists);
        Line::styled(
            format!("[{}]", assists_text),
            Style::default().fg(Color::DarkGray),
        )
        .alignment(alignment)
    } else {
        Line::styled("[Unassisted]", Style::default().fg(Color::DarkGray)).alignment(alignment)
    }
}

fn get_assists_text(assists: &Vec<AssistInfo>) -> String {
    assists
        .iter()
        .map(|assist| {
            format!(
                "{} ({})",
                assist.name,
                assist.assists_to_date.unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn get_period_title(period: &PeriodDescriptor) -> String {
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
