use chrono_tz::Tz;
use ratatui::layout::{Direction, Layout};
use ratatui::widgets::Paragraph;

use crate::app::App;
use crate::models::{
    games::games::{GameState, PeriodType},
    playoffs::series::SeriesResponse,
};
use crate::ui::{
    games::games::{get_period_title, split_info_left_middle_right},
    games::stats::{AWAY_BAR_COLOR, HOME_BAR_COLOR},
    render::{border_style, split_area_vertical},
};

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Style},
    text::Line,
    widgets::Block,
};

use tui_big_text::{BigText, PixelSize};

use crate::ui::games::games::MIDDLE_LENGTH;

pub fn render_series(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::bordered()
        .title(format!(
            " {} Stanley Cup Playoffs ",
            app.state.date_state.year
        ))
        .border_style(border_style());
    let inner = block.inner(area);

    frame.render_widget(block, area);
    let upper_score_schedule = split_area_vertical(
        inner,
        [
            Constraint::Length(4), // upper info (1 at bottom for spacing)
            Constraint::Length(4), // series score (big text)
            Constraint::Fill(1),   // games schedule
        ],
    );

    if let Some(series) = &app.state.playoffs.series_data {
        // Upper info
        let upper_info_chunks = split_area_vertical(
            upper_score_schedule[0],
            [
                Constraint::Length(1), // Round information
                Constraint::Length(1), // Spacing
                Constraint::Length(1), // Teams
            ],
        );
        // Round info
        let round_info = match series.round_abbrev.as_str() {
            "R1" => "Round 1",
            "R2" => "Round 2",
            "CF" => "Conference Finals",
            "QF" => "Quarter Finals",
            "SF" => "Semi Finals",
            "ECF" => "Eastern Conference Finals",
            "WCF" => "Western Conference Finals",
            "SCF" => "Stanley Cup Final",
            _ => &series.round_abbrev.clone(),
        };
        let round_info_line = Line::from(round_info).centered();
        frame.render_widget(round_info_line, upper_info_chunks[0]);
        // Teams
        let teams_chunks = split_info_left_middle_right(upper_info_chunks[2], MIDDLE_LENGTH);
        let bottom_seed = if series.bottom_seed_team.id == -1 {
            Line::from("TBD").style(Style::new().fg(Color::DarkGray))
        } else {
            Line::from(format!(
                "{} {}",
                series.bottom_seed_team.place_name.default, series.bottom_seed_team.name.default
            ))
        };
        frame.render_widget(bottom_seed.right_aligned(), teams_chunks[0]);
        frame.render_widget(Line::from("vs").centered(), teams_chunks[1]);
        let top_seed = if series.top_seed_team.id == -1 {
            Line::from("TBD").style(Style::new().fg(Color::DarkGray))
        } else {
            Line::from(format!(
                "{} {}",
                series.top_seed_team.place_name.default, series.top_seed_team.name.default
            ))
        };
        frame.render_widget(top_seed, teams_chunks[2]);

        render_big_series_score(series, frame, upper_score_schedule[1]);

        render_schedule(
            series,
            frame,
            upper_score_schedule[2],
            app.settings.timezone,
            &app.settings.timezone_abbreviation,
            app.state.playoffs.vertical_scroll_offset,
            &mut app.state.playoffs.vertical_max_scroll,
            &mut app.state.playoffs.visible_rows,
        );
    }
}

fn render_big_series_score(series: &SeriesResponse, frame: &mut Frame, area: Rect) {
    let chunks = split_info_left_middle_right(area, MIDDLE_LENGTH);

    let bottom_seed_score = build_big_text(
        series.bottom_seed_team.series_wins.to_string(),
        Alignment::Right,
    );
    frame.render_widget(bottom_seed_score, chunks[0]);
    let dash = build_big_text("-".to_string(), Alignment::Center);
    frame.render_widget(dash, chunks[1]);
    let top_seed_score = build_big_text(
        series.top_seed_team.series_wins.to_string(),
        Alignment::Left,
    );
    frame.render_widget(top_seed_score, chunks[2]);
}

fn build_big_text(text: String, alignment: Alignment) -> BigText<'static> {
    BigText::builder()
        .pixel_size(PixelSize::Sextant)
        .style(Style::new().fg(HOME_BAR_COLOR))
        .lines(vec![Line::from(text)])
        .alignment(alignment)
        .build()
}

fn render_schedule(
    series: &SeriesResponse,
    frame: &mut Frame,
    area: Rect,
    timezone: Tz,
    timezone_abbr: &str,
    scroll_offset: usize,
    max_scroll: &mut usize,
    visible_rows: &mut usize,
) {
    *visible_rows = area.height.saturating_sub(3) as usize;

    let mut game_number_lines = vec![];
    let mut away_team_lines = vec![];
    let mut away_score_lines = vec![];
    let mut game_status_lines = vec![];
    let mut home_score_lines = vec![];
    let mut home_team_lines = vec![];

    let mut has_if_necessary = false;

    for game in &series.games {
        if game.if_necessary {
            has_if_necessary = true;
        }
        let game_label = format!(
            "{}Game {}:",
            if game.if_necessary { "*" } else { "" },
            game.game_number
        );
        game_number_lines.push(Line::from(game_label));

        let date = game
            .compute_local_time(timezone)
            .format("%b %d")
            .to_string();
        game_number_lines.push(Line::from(date).style(Style::new().fg(Color::DarkGray)));

        let away_score = game
            .away_team
            .score
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string());
        let home_score = game
            .home_team
            .score
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string());

        away_score_lines.push(Line::from(away_score).centered());
        away_score_lines.push(Line::default());

        home_score_lines.push(Line::from(home_score).centered());
        home_score_lines.push(Line::default());

        let status_line = match game.game_state {
            GameState::FUT | GameState::PRE => Line::from(format!(
                "{} {}",
                game.compute_local_time(timezone).format("%-I:%M %p"),
                timezone_abbr
            )),

            GameState::LIVE | GameState::CRIT => match game.period_descriptor.as_ref() {
                None => Line::from("Live"),
                Some(d) => Line::styled(get_period_title(d), Style::new().fg(Color::Green)),
            },

            GameState::OVER | GameState::FINAL | GameState::OFF => {
                let outcome = game.game_outcome.as_ref().unwrap();
                Line::from(match outcome.last_period_type {
                    PeriodType::REG | PeriodType::Unknown => "Final".to_string(),
                    PeriodType::OT => match outcome.ot_periods.unwrap_or(0) {
                        n if n > 1 => format!("Final/{}OT", n),
                        _ => "Final/OT".to_string(),
                    },
                    PeriodType::SO => "Final/SO".to_string(),
                })
            }

            GameState::Unknown => Line::default(),
        };

        game_status_lines.push(status_line.centered());
        game_status_lines.push(Line::default());

        let winner = match (game.away_team.score, game.home_team.score) {
            (Some(a), Some(h)) if a > h => Some("away"),
            (Some(a), Some(h)) if h > a => Some("home"),
            _ => None,
        };

        let is_final = matches!(
            game.game_state,
            GameState::OVER | GameState::FINAL | GameState::OFF
        );

        let away_is_top = (series.top_seed_team.id != -1)
            && (game.away_team.id == series.top_seed_team.id as u32);
        let away_base_color = if away_is_top {
            AWAY_BAR_COLOR
        } else {
            HOME_BAR_COLOR
        };

        let is_played = matches!(
            game.game_state,
            GameState::OVER | GameState::FINAL | GameState::OFF
        );

        let away_style = if is_played {
            match (winner, is_final) {
                (Some("away"), true) => Style::new().fg(away_base_color).bold(),

                (_, true) => Style::new().fg(Color::DarkGray),

                _ => Style::new().fg(away_base_color),
            }
        } else {
            Style::default()
        };

        let away_team_line = Line::styled(game.away_team.common_name.default.clone(), away_style);

        away_team_lines.push(away_team_line);
        away_team_lines.push(Line::default());

        let home_is_top = (series.top_seed_team.id != -1)
            && (game.home_team.id == series.top_seed_team.id as u32);

        let home_base_color = if home_is_top {
            AWAY_BAR_COLOR
        } else {
            HOME_BAR_COLOR
        };

        let home_style = if is_played {
            match (winner, is_final) {
                (Some("home"), true) => Style::new().fg(home_base_color).bold(),

                (_, true) => Style::new().fg(Color::DarkGray),

                _ => Style::new().fg(home_base_color),
            }
        } else {
            Style::default()
        };

        let home_team_line = Line::styled(
            format!("   {}", game.home_team.common_name.default),
            home_style,
        );

        home_team_lines.push(home_team_line);
        home_team_lines.push(Line::default());

        // Empty line for spacing
        game_number_lines.push(Line::default());
        away_score_lines.push(Line::default());
        home_score_lines.push(Line::default());
        game_status_lines.push(Line::default());
        away_team_lines.push(Line::default());
        home_team_lines.push(Line::default());
    }

    let vert_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let content_height = vert_chunks[1].height as usize;

    let max_scroll_val = game_number_lines.len().saturating_sub(content_height);
    *max_scroll = max_scroll_val;

    let offset = scroll_offset.min(max_scroll_val);

    let can_scroll_up = offset > 0;
    let can_scroll_down = offset < max_scroll_val;

    let end = (offset + content_height).min(game_number_lines.len());

    let visible_game_number_lines = game_number_lines[offset..end].to_vec();
    let visible_away_team_lines = away_team_lines[offset..end].to_vec();
    let visible_away_score_lines = away_score_lines[offset..end].to_vec();
    let visible_game_status_lines = game_status_lines[offset..end].to_vec();
    let visible_home_score_lines = home_score_lines[offset..end].to_vec();
    let visible_home_team_lines = home_team_lines[offset..end].to_vec();

    frame.render_widget(
        Line::from(if can_scroll_up { "▲" } else { "" }).centered(),
        vert_chunks[0],
    );

    frame.render_widget(
        Line::from(if can_scroll_down { "▼" } else { "" }).centered(),
        vert_chunks[2],
    );

    let constraints = [
        Constraint::Fill(1),
        Constraint::Length(1), // For spacing
        Constraint::Length(9),
        Constraint::Length(14),
        Constraint::Length(3),
        Constraint::Length(16),
        Constraint::Length(3),
        Constraint::Length(14),
        Constraint::Fill(1),
    ];

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(vert_chunks[1]);

    frame.render_widget(Paragraph::new(visible_game_number_lines), columns[2]);
    frame.render_widget(Paragraph::new(visible_away_team_lines), columns[3]);
    frame.render_widget(Paragraph::new(visible_away_score_lines), columns[4]);
    frame.render_widget(Paragraph::new(visible_game_status_lines), columns[5]);
    frame.render_widget(Paragraph::new(visible_home_score_lines), columns[6]);
    frame.render_widget(Paragraph::new(visible_home_team_lines), columns[7]);

    if has_if_necessary {
        frame.render_widget(
            Line::from("* If necessary").style(Style::new().fg(Color::DarkGray)),
            columns[0],
        );
    }
}
