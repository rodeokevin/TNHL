use crate::App;
use crate::models::games::game_story::{GameStatsCategory, StatValue};
use crate::ui::{
    games::{games::split_info_left_middle_right, scoring::MIDDLE_LENGTH},
    render::BORDER_FOCUSED_COLOR,
};
use std::collections::HashMap;
use std::vec;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

const AWAY_BAR_COLOR: Color = Color::Rgb(220, 50, 47); // Red
const HOME_BAR_COLOR: Color = Color::Rgb(38, 139, 210); // Blue

pub fn render_stats(frame: &mut Frame, app: &mut App, area: Rect) {
    let game_story = app
        .state
        .games
        .games_data
        .as_ref()
        .and_then(|g| g.games.get(app.state.games.selected_game_index))
        .and_then(|g| app.state.games.game_story_data.get(&g.id));

    let mut away_lines = vec![];
    let mut middle_lines = vec![];
    let mut home_lines = vec![];

    if let Some(game_story) = game_story
        && let Some(summary) = &game_story.summary
        && !summary.team_game_stats.is_empty()
    {
        let stats_map: HashMap<_, _> = summary
            .team_game_stats
            .iter()
            .map(|stat| (stat.category.clone(), stat))
            .collect();

        // Shots on goal
        if let Some(sog) = stats_map.get(&GameStatsCategory::Sog) {
            middle_lines.push(
                Line::from("Shots on Goal")
                    .style(Style::default().fg(BORDER_FOCUSED_COLOR))
                    .alignment(Alignment::Center),
            );
            away_lines.push(Line::default());
            home_lines.push(Line::default());
            away_lines.push(Line::from(sog.away_value.to_string()).alignment(Alignment::Right));
            middle_lines.push(compute_middle_bar(&sog.away_value, &sog.home_value));
            home_lines.push(Line::from(sog.home_value.to_string()).alignment(Alignment::Left));
        }
        // Face-off %
        if let Some(&faceoff) = stats_map.get(&GameStatsCategory::FaceoffWinningPctg) {
            middle_lines.push(
                Line::from("Face-off %")
                    .style(Style::default().fg(BORDER_FOCUSED_COLOR))
                    .alignment(Alignment::Center),
            );
            away_lines.push(Line::default());
            home_lines.push(Line::default());
            away_lines.push(
                Line::from(format!("{}%", faceoff.away_value.to_string()))
                    .alignment(Alignment::Right),
            );
            middle_lines.push(compute_middle_bar(&faceoff.away_value, &faceoff.home_value));
            home_lines.push(
                Line::from(format!("{}%", faceoff.home_value.to_string()))
                    .alignment(Alignment::Left),
            );
        }
        // Power Play %
        if let Some(&power_play_pctg) = stats_map.get(&GameStatsCategory::PowerPlayPctg) {
            middle_lines.push(
                Line::from("Power Play %")
                    .style(Style::default().fg(BORDER_FOCUSED_COLOR))
                    .alignment(Alignment::Center),
            );
            away_lines.push(Line::default());
            home_lines.push(Line::default());
            away_lines.push(
                Line::from(format!("{}%", power_play_pctg.away_value.to_string()))
                    .alignment(Alignment::Right),
            );
            middle_lines.push(compute_middle_bar(
                &power_play_pctg.away_value,
                &power_play_pctg.home_value,
            ));
            home_lines.push(
                Line::from(format!("{}%", power_play_pctg.home_value.to_string()))
                    .alignment(Alignment::Left),
            );
            if let Some(&power_play_rate) = stats_map.get(&GameStatsCategory::PowerPlay) {
                away_lines.push(
                    Line::from(power_play_rate.away_value.to_string())
                        .style(Style::default().fg(Color::DarkGray))
                        .alignment(Alignment::Right),
                );
                middle_lines.push(Line::default().alignment(Alignment::Center));
                home_lines.push(
                    Line::from(power_play_rate.home_value.to_string())
                        .style(Style::default().fg(Color::DarkGray))
                        .alignment(Alignment::Left),
                );
            }
        }
        // Penalty minues
        if let Some(&pims) = stats_map.get(&GameStatsCategory::Pim) {
            middle_lines.push(
                Line::from("Penalty Minutes")
                    .style(Style::default().fg(BORDER_FOCUSED_COLOR))
                    .alignment(Alignment::Center),
            );
            away_lines.push(Line::default());
            home_lines.push(Line::default());
            away_lines.push(Line::from(pims.away_value.to_string()).alignment(Alignment::Right));
            middle_lines.push(compute_middle_bar(&pims.away_value, &pims.home_value));
            home_lines.push(Line::from(pims.home_value.to_string()).alignment(Alignment::Left));
        }
        // Hits
        if let Some(&hits) = stats_map.get(&GameStatsCategory::Hits) {
            middle_lines.push(
                Line::from("Hits")
                    .style(Style::default().fg(BORDER_FOCUSED_COLOR))
                    .alignment(Alignment::Center),
            );
            away_lines.push(Line::default());
            home_lines.push(Line::default());
            away_lines.push(Line::from(hits.away_value.to_string()).alignment(Alignment::Right));
            middle_lines.push(compute_middle_bar(&hits.away_value, &hits.home_value));
            home_lines.push(Line::from(hits.home_value.to_string()).alignment(Alignment::Left));
        }
        // Blocked Shots
        if let Some(&blocked_shots) = stats_map.get(&GameStatsCategory::BlockedShots) {
            middle_lines.push(
                Line::from("Blocked Shots")
                    .style(Style::default().fg(BORDER_FOCUSED_COLOR))
                    .alignment(Alignment::Center),
            );
            away_lines.push(Line::default());
            home_lines.push(Line::default());
            away_lines
                .push(Line::from(blocked_shots.away_value.to_string()).alignment(Alignment::Right));
            middle_lines.push(compute_middle_bar(
                &blocked_shots.away_value,
                &blocked_shots.home_value,
            ));
            home_lines
                .push(Line::from(blocked_shots.home_value.to_string()).alignment(Alignment::Left));
        }
        // Giveaways
        if let Some(&giveaways) = stats_map.get(&GameStatsCategory::Giveaways) {
            middle_lines.push(
                Line::from("Giveaways")
                    .style(Style::default().fg(BORDER_FOCUSED_COLOR))
                    .alignment(Alignment::Center),
            );
            away_lines.push(Line::default());
            home_lines.push(Line::default());
            away_lines
                .push(Line::from(giveaways.away_value.to_string()).alignment(Alignment::Right));
            middle_lines.push(compute_middle_bar(
                &giveaways.away_value,
                &giveaways.home_value,
            ));
            home_lines
                .push(Line::from(giveaways.home_value.to_string()).alignment(Alignment::Left));
        }
        // Takeaways
        if let Some(&takeaways) = stats_map.get(&GameStatsCategory::Takeaways) {
            middle_lines.push(
                Line::from("Takeaways")
                    .style(Style::default().fg(BORDER_FOCUSED_COLOR))
                    .alignment(Alignment::Center),
            );
            away_lines.push(Line::default());
            home_lines.push(Line::default());
            away_lines
                .push(Line::from(takeaways.away_value.to_string()).alignment(Alignment::Right));

            middle_lines.push(compute_middle_bar(
                &takeaways.away_value,
                &takeaways.home_value,
            ));
            home_lines
                .push(Line::from(takeaways.home_value.to_string()).alignment(Alignment::Left));
        }
    } else {
        // No stats
        away_lines.push(Line::default());
        middle_lines.push(
            Line::from("No stats yet")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center),
        );
        home_lines.push(Line::default());
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
    app.state.games.max_scroll = last_line;
    let offset = app.state.games.scroll_offset.min(last_line);
    let can_scroll_up = offset > 0;
    let can_scroll_down = offset < last_line;

    // Slice to visible window
    let end = (offset + content_height).min(away_lines.len());

    let visible_away: Vec<Line> = away_lines[offset..end].to_vec();
    let visible_home: Vec<Line> = home_lines[offset..end].to_vec();
    let visible_period: Vec<Line> = middle_lines[offset..end].to_vec();

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

fn compute_middle_bar<'a>(away_value: &'a StatValue, home_value: &'a StatValue) -> Line<'static> {
    let mut away_length = 0;
    let mut home_length = 0;
    let (away_zero, home_zero) = (away_value.is_zero(), home_value.is_zero());
    if away_zero && home_zero {
        away_length = (MIDDLE_LENGTH - 3) / 2;
        home_length = (MIDDLE_LENGTH - 3) / 2;
    } else if away_zero {
        home_length = MIDDLE_LENGTH - 2;
    } else if home_zero {
        away_length = MIDDLE_LENGTH - 2;
    } else {
        let total = (MIDDLE_LENGTH - 3) as f64;
        let away = match away_value {
            StatValue::Int(v) => *v as f64,
            StatValue::Float(v) => *v,
            _ => 0.0,
        };
        let home = match home_value {
            StatValue::Int(v) => *v as f64,
            StatValue::Float(v) => *v,
            _ => 0.0,
        };
        let sum = away + home;
        away_length = ((away / sum) * total).round().max(1.0) as u16;
        home_length = MIDDLE_LENGTH - 3 - away_length;
    }
    let gap = std::iter::once(Span::raw(if away_zero ^ home_zero { "" } else { " " }));
    let away_spans: Vec<_> =
        std::iter::repeat(Span::styled("─", Style::default().fg(AWAY_BAR_COLOR)))
            .take(away_length as usize)
            .collect();
    let home_spans: Vec<_> =
        std::iter::repeat(Span::styled("─", Style::default().fg(HOME_BAR_COLOR)))
            .take(home_length as usize)
            .collect();
    let spans: Vec<_> = away_spans
        .into_iter()
        .chain(gap)
        .chain(home_spans.into_iter())
        .collect();

    Line::from(spans).alignment(Alignment::Center)
}
