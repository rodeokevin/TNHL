use crate::App;
use crate::models::games::game_story::{GameStatsCategory, StatValue, TeamGameStats};
use crate::ui::{
    games::{games::split_info_left_middle_right, scoring::MIDDLE_LENGTH},
    render::BORDER_FOCUSED_COLOR,
};
use std::collections::HashMap;
use std::vec;

use ratatui::style::Modifier;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

pub const AWAY_BAR_COLOR: Color = Color::Rgb(220, 50, 47); // Red
pub const HOME_BAR_COLOR: Color = Color::Rgb(38, 139, 210); // Blue

pub fn render_stats(frame: &mut Frame, app: &mut App, area: Rect) {
    // Pass visible rows to game state
    app.state.games.visible_rows = area.height.saturating_sub(3) as usize;

    let game_id = app
        .state
        .games
        .games_data
        .as_ref()
        .and_then(|g| g.games.get(app.state.games.selected_game_index))
        .map(|g| g.id);
    let game_story = game_id.and_then(|id| app.state.games.game_story_data.get(&id));

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
        if let Some(&sog) = stats_map.get(&GameStatsCategory::Sog) {
            add_stat_lines(
                &mut away_lines,
                &mut middle_lines,
                &mut home_lines,
                "Shots on Goal".to_string(),
                sog,
                sog.away_value.to_string(),
                sog.home_value.to_string(),
            );
        }
        // Face-off %
        if let Some(&faceoff) = stats_map.get(&GameStatsCategory::FaceoffWinningPctg) {
            add_stat_lines(
                &mut away_lines,
                &mut middle_lines,
                &mut home_lines,
                "Face-off %".to_string(),
                faceoff,
                format!("{}%", faceoff.away_value.to_string()),
                format!("{}%", faceoff.home_value.to_string()),
            );
        }
        // Power Play %
        if let Some(&power_play_pctg) = stats_map.get(&GameStatsCategory::PowerPlayPctg) {
            add_stat_lines(
                &mut away_lines,
                &mut middle_lines,
                &mut home_lines,
                "Power Play %".to_string(),
                power_play_pctg,
                format!("{}%", power_play_pctg.away_value.to_string()),
                format!("{}%", power_play_pctg.home_value.to_string()),
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
        // Penalty minutes
        if let Some(&pims) = stats_map.get(&GameStatsCategory::Pim) {
            add_stat_lines(
                &mut away_lines,
                &mut middle_lines,
                &mut home_lines,
                "Power Play %".to_string(),
                pims,
                pims.away_value.to_string(),
                pims.home_value.to_string(),
            );
        }
        // Hits
        if let Some(&hits) = stats_map.get(&GameStatsCategory::Hits) {
            add_stat_lines(
                &mut away_lines,
                &mut middle_lines,
                &mut home_lines,
                "Hits".to_string(),
                hits,
                hits.away_value.to_string(),
                hits.home_value.to_string(),
            );
        }
        // Blocked Shots
        if let Some(&blocked_shots) = stats_map.get(&GameStatsCategory::BlockedShots) {
            add_stat_lines(
                &mut away_lines,
                &mut middle_lines,
                &mut home_lines,
                "Blocked Shots".to_string(),
                blocked_shots,
                blocked_shots.away_value.to_string(),
                blocked_shots.home_value.to_string(),
            );
        }
        // Giveaways
        if let Some(&giveaways) = stats_map.get(&GameStatsCategory::Giveaways) {
            add_stat_lines(
                &mut away_lines,
                &mut middle_lines,
                &mut home_lines,
                "Giveaways".to_string(),
                giveaways,
                giveaways.away_value.to_string(),
                giveaways.home_value.to_string(),
            );
        }
        // Takeaways
        if let Some(&takeaways) = stats_map.get(&GameStatsCategory::Takeaways) {
            add_stat_lines(
                &mut away_lines,
                &mut middle_lines,
                &mut home_lines,
                "Takeaways".to_string(),
                takeaways,
                takeaways.away_value.to_string(),
                takeaways.home_value.to_string(),
            );
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

// Add the lines for one stat
fn add_stat_lines(
    away_lines: &mut Vec<Line<'_>>,
    middle_lines: &mut Vec<Line<'_>>,
    home_lines: &mut Vec<Line<'_>>,
    title: String,
    stat: &TeamGameStats,
    away_value: String,
    home_value: String,
) {
    away_lines.push(Line::default());
    middle_lines.push(
        Line::from(title)
            .style(
                Style::default()
                    .fg(BORDER_FOCUSED_COLOR)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center),
    );
    home_lines.push(Line::default());

    away_lines.push(Line::from(away_value).alignment(Alignment::Right));
    middle_lines.push(compute_middle_bar(&stat.away_value, &stat.home_value));
    home_lines.push(Line::from(home_value).alignment(Alignment::Left));
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
