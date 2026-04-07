use crate::App;
use crate::models::game_story::{GameStatsCategory, GameStoryReponse, TeamGameStats};
use crate::ui::render::BORDER_FOCUSED_COLOR;
use std::collections::HashMap;
use std::rc::Rc;
use std::vec;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::Paragraph,
};

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
            away_lines.push(Line::from(""));
            home_lines.push(Line::from(""));
            away_lines.push(Line::from(sog.away_value.to_string()).alignment(Alignment::Right));
            middle_lines.push(Line::from("---").alignment(Alignment::Center));
            home_lines.push(Line::from(sog.home_value.to_string()).alignment(Alignment::Left));
        }
        // Face-off %
        if let Some(faceoff) = stats_map.get(&GameStatsCategory::FaceoffWinningPctg) {
            middle_lines.push(
                Line::from("Face-off %")
                    .style(Style::default().fg(BORDER_FOCUSED_COLOR))
                    .alignment(Alignment::Center),
            );
            away_lines.push(Line::from(""));
            home_lines.push(Line::from(""));
            away_lines.push(
                Line::from(format!("{}%", faceoff.away_value.to_string()))
                    .alignment(Alignment::Right),
            );
            middle_lines.push(Line::from("---").alignment(Alignment::Center));
            home_lines.push(
                Line::from(format!("{}%", faceoff.home_value.to_string()))
                    .alignment(Alignment::Left),
            );
        }
        // Power Play %
        if let Some(power_play_pctg) = stats_map.get(&GameStatsCategory::PowerPlayPctg) {
            middle_lines.push(
                Line::from("Power Play %")
                    .style(Style::default().fg(BORDER_FOCUSED_COLOR))
                    .alignment(Alignment::Center),
            );
            away_lines.push(Line::from(""));
            home_lines.push(Line::from(""));
            away_lines.push(
                Line::from(format!("{}%", power_play_pctg.away_value.to_string()))
                    .alignment(Alignment::Right),
            );
            middle_lines.push(Line::from("---").alignment(Alignment::Center));
            home_lines.push(
                Line::from(format!("{}%", power_play_pctg.home_value.to_string()))
                    .alignment(Alignment::Left),
            );
            if let Some(power_play_rate) = stats_map.get(&GameStatsCategory::PowerPlay) {
                away_lines.push(
                    Line::from(power_play_rate.away_value.to_string())
                        .style(Style::default().fg(Color::DarkGray))
                        .alignment(Alignment::Right),
                );
                middle_lines.push(Line::from("").alignment(Alignment::Center));
                home_lines.push(
                    Line::from(power_play_rate.home_value.to_string())
                        .style(Style::default().fg(Color::DarkGray))
                        .alignment(Alignment::Left),
                );
            }
        }
        // Penalty minues
        if let Some(pims) = stats_map.get(&GameStatsCategory::Pim) {
            middle_lines.push(
                Line::from("Penalty Minutes")
                    .style(Style::default().fg(BORDER_FOCUSED_COLOR))
                    .alignment(Alignment::Center),
            );
            away_lines.push(Line::from(""));
            home_lines.push(Line::from(""));
            away_lines.push(Line::from(pims.away_value.to_string()).alignment(Alignment::Right));
            middle_lines.push(Line::from("---").alignment(Alignment::Center));
            home_lines.push(Line::from(pims.home_value.to_string()).alignment(Alignment::Left));
        }
        // Hits
        if let Some(hits) = stats_map.get(&GameStatsCategory::Hits) {
            middle_lines.push(
                Line::from("Hits")
                    .style(Style::default().fg(BORDER_FOCUSED_COLOR))
                    .alignment(Alignment::Center),
            );
            away_lines.push(Line::from(""));
            home_lines.push(Line::from(""));
            away_lines.push(Line::from(hits.away_value.to_string()).alignment(Alignment::Right));
            middle_lines.push(Line::from("---").alignment(Alignment::Center));
            home_lines.push(Line::from(hits.home_value.to_string()).alignment(Alignment::Left));
        }
        // Blocked Shots
        if let Some(blocked_shots) = stats_map.get(&GameStatsCategory::BlockedShots) {
            middle_lines.push(
                Line::from("Blocked Shots")
                    .style(Style::default().fg(BORDER_FOCUSED_COLOR))
                    .alignment(Alignment::Center),
            );
            away_lines.push(Line::from(""));
            home_lines.push(Line::from(""));
            away_lines
                .push(Line::from(blocked_shots.away_value.to_string()).alignment(Alignment::Right));
            middle_lines.push(Line::from("---").alignment(Alignment::Center));
            home_lines
                .push(Line::from(blocked_shots.home_value.to_string()).alignment(Alignment::Left));
        }
        // Giveaways
        if let Some(giveaways) = stats_map.get(&GameStatsCategory::Giveaways) {
            middle_lines.push(
                Line::from("Giveaways")
                    .style(Style::default().fg(BORDER_FOCUSED_COLOR))
                    .alignment(Alignment::Center),
            );
            away_lines.push(Line::from(""));
            home_lines.push(Line::from(""));
            away_lines
                .push(Line::from(giveaways.away_value.to_string()).alignment(Alignment::Right));
            middle_lines.push(Line::from("---").alignment(Alignment::Center));
            home_lines
                .push(Line::from(giveaways.home_value.to_string()).alignment(Alignment::Left));
        }
        // Takeaways
        if let Some(takeaways) = stats_map.get(&GameStatsCategory::Takeaways) {
            middle_lines.push(
                Line::from("Takeaways")
                    .style(Style::default().fg(BORDER_FOCUSED_COLOR))
                    .alignment(Alignment::Center),
            );
            away_lines.push(Line::from(""));
            home_lines.push(Line::from(""));
            away_lines
                .push(Line::from(takeaways.away_value.to_string()).alignment(Alignment::Right));
            middle_lines.push(Line::from("---").alignment(Alignment::Center));
            home_lines
                .push(Line::from(takeaways.home_value.to_string()).alignment(Alignment::Left));
        }
    } else {
        // No stats
        away_lines.push(Line::from(""));
        middle_lines.push(Line::from("No stats yet").style(Style::default().fg(Color::DarkGray)));
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
    let last_line = away_lines.len().saturating_sub(content_height);
    app.state.games.max_scroll = last_line;
    let offset = app.state.games.scroll_offset.min(last_line);
    let can_scroll_up = offset > 0;
    let can_scroll_down = offset < last_line;

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

fn get_stat(
    stats_map: &HashMap<GameStatsCategory, &TeamGameStats>,
    category: GameStatsCategory,
) -> (String, String) {
    stats_map
        .get(&category)
        .map(|stat| (stat.away_value.to_string(), stat.home_value.to_string()))
        .unwrap_or_else(|| ("--".to_string(), "--".to_string()))
}
