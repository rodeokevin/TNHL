use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::models::playoffs::bracket::Series;
use crate::ui::render::border_style;
use crate::{app::App, state::playoffs_state::PlayoffsState};

// Base card width used for most brackets.
const CARD_WIDTH: u16 = 18;
// Wider card width used for division-era brackets that show a seed prefix
// (e.g. "WC1 ") on round-1 cards. Uniform across the whole bracket.
const WIDE_CARD_WIDTH: u16 = 22;
const CARD_HEIGHT: u16 = 5;
// Horizontal gap length between rounds
const ROUND_HOR_GAP: u16 = 6;

/// Card width for a given bracket year. Division-era years (2014+, except the
/// 2020 COVID bubble which used conference-wide reseeding) show a seed prefix
/// on round-1 cards, so those brackets use the wider card.
fn card_width_for_year(year: i32) -> u16 {
    if year >= 2014 && year != 2020 {
        WIDE_CARD_WIDTH
    } else {
        CARD_WIDTH
    }
}

const COLOR_WIN: Color = Color::Green;
const COLOR_LOSE: Color = Color::DarkGray;

pub fn render_playoffs(frame: &mut Frame, app: &mut App, area: Rect) {
    let outer_block = Block::bordered()
        .border_style(border_style())
        .title(format!(
            " {} Stanley Cup Playoffs ",
            app.state.date_state.year
        ));

    let inner = outer_block.inner(area);

    frame.render_widget(outer_block, area);

    if let Some(playoff_bracket) = &app.state.playoffs.bracket_data {
        let h_off = app.state.playoffs.horizontal_scroll_offset as u16;
        let v_off = app.state.playoffs.vertical_scroll_offset as u16;
        let year = app.state.date_state.year;
        let cw = card_width_for_year(year);

        let bracket_area = Rect {
            x: inner.x + 1,
            y: inner.y + 1,
            width: inner.width.saturating_sub(2),
            height: inner.height.saturating_sub(2),
        };
        // Pass visible rows/columns and max scroll to playoff_bracket state
        app.state.playoffs.visible_columns = bracket_area.width.saturating_sub(1) as usize;
        app.state.playoffs.visible_rows = bracket_area.height.saturating_sub(1) as usize;

        app.state.playoffs.horizontal_max_scroll =
            canvas_width(cw).saturating_sub(bracket_area.width) as usize;
        app.state.playoffs.vertical_max_scroll =
            canvas_height().saturating_sub(bracket_area.height) as usize;

        render_bracket(
            frame,
            bracket_area,
            &playoff_bracket.series,
            year,
            h_off,
            v_off,
        );
        render_scroll_indicators(frame, inner, &app.state.playoffs);
    };
}

// 6 columns, 4 rows
fn series_letter_to_position(letter: &str) -> Option<(usize, usize)> {
    match letter {
        "A" => Some((6, 0)), // Top right
        "B" => Some((6, 1)),
        "C" => Some((6, 2)),
        "D" => Some((6, 3)),
        "I" => Some((5, 0)),
        "J" => Some((5, 1)),
        "M" => Some((4, 0)),
        "O" => Some((3, 0)), // Stanley Cup Final
        "N" => Some((2, 0)),
        "K" => Some((1, 0)),
        "L" => Some((1, 1)),
        "E" => Some((0, 0)), // Top left
        "F" => Some((0, 1)),
        "G" => Some((0, 2)),
        "H" => Some((0, 3)),
        _ => None,
    }
}

const COLUMN_LABELS: [&str; 7] = ["R1", "R2", "WCF", "SCF", "ECF", "R2", "R1"];

fn canvas_width(cw: u16) -> u16 {
    7 * cw + 6 * ROUND_HOR_GAP
}
fn canvas_height() -> u16 {
    1 + 4 * CARD_HEIGHT + 3 * 1 // label row + 4 cards + 3 gaps of 1
}

fn r1_y(row: usize) -> u16 {
    let gap = 1;
    1 + row as u16 * (CARD_HEIGHT + gap)
}
fn midpoint(a: u16, b: u16) -> u16 {
    (a + b) / 2
}

// Compute the card position based on R1 cards
fn card_virtual_pos(col: usize, row: usize, cw: u16) -> (u16, u16) {
    let x = col as u16 * (cw + ROUND_HOR_GAP);

    let y = match col {
        // R1
        0 | 6 => r1_y(row),
        // R2
        1 | 5 => match row {
            0 => midpoint(r1_y(0), r1_y(1)),
            1 => midpoint(r1_y(2), r1_y(3)),
            _ => 0,
        },
        // Conference finals
        2 | 4 => midpoint(midpoint(r1_y(0), r1_y(1)), midpoint(r1_y(2), r1_y(3))),
        // Stanley Cup Final
        3 => midpoint(
            midpoint(midpoint(r1_y(0), r1_y(1)), midpoint(r1_y(2), r1_y(3))),
            midpoint(midpoint(r1_y(0), r1_y(1)), midpoint(r1_y(2), r1_y(3))),
        ),
        _ => 0,
    };

    (x, y)
}

fn card_mid_y(col: usize, row: usize, cw: u16) -> u16 {
    card_virtual_pos(col, row, cw).1 + CARD_HEIGHT / 2
}

fn render_bracket(
    frame: &mut Frame,
    area: Rect,
    series_list: &[Series],
    year: i32,
    h_off: u16,
    v_off: u16,
) {
    let cw = card_width_for_year(year);

    // Column labels
    for (col, label) in COLUMN_LABELS.iter().enumerate() {
        let vx = col as u16 * (cw + ROUND_HOR_GAP);
        draw_round_label(frame, area, vx, 0, cw, label, h_off, v_off);
    }

    // Series cards — look up each series' column/row from its letter
    for series in series_list {
        let Some((col, row)) = series_letter_to_position(&series.series_letter) else {
            continue;
        };
        let (vx, vy) = card_virtual_pos(col, row, cw);
        render_series_card(frame, area, series, year, cw, vx, vy, h_off, v_off);
    }

    // Connectors
    draw_east_connectors(frame, area, cw, h_off, v_off);
    draw_west_connectors(frame, area, cw, h_off, v_off);
}

fn draw_round_label(
    frame: &mut Frame,
    area: Rect,
    vx: u16,
    vy: u16,
    width: u16,
    label: &str,
    h_off: u16,
    v_off: u16,
) {
    let ax = vx as i32 - h_off as i32;
    let ay = vy as i32 - v_off as i32;
    if ax + width as i32 <= 0 || ax >= area.width as i32 || ay < 0 || ay >= area.height as i32 {
        return;
    }
    let x = (area.x as i32 + ax).max(area.x as i32) as u16;
    let w = (width as i32 - (0 - ax).max(0))
        .max(0)
        .min((area.x as i32 + area.width as i32 - x as i32).max(0)) as u16;
    if w == 0 {
        return;
    }
    frame.render_widget(
        Line::from(label.to_string()).centered(),
        Rect {
            x,
            y: area.y + ay as u16,
            width: w,
            height: 1,
        },
    );
}

/// Render the series card
/// Computes actual positions based on scrolling offsets
/// If the card goes outside of the render area, it clips
fn render_series_card(
    frame: &mut Frame,
    area: Rect,
    series: &Series,
    year: i32,
    cw: u16,
    vx: u16,
    vy: u16,
    h_off: u16,
    v_off: u16,
) {
    let ax = vx as i32 - h_off as i32;
    let ay = vy as i32 - v_off as i32;

    if ax + cw as i32 <= 0
        || ax >= area.width as i32
        || ay + CARD_HEIGHT as i32 <= 0
        || ay >= area.height as i32
    {
        return;
    }

    let left = ax;
    let right = ax + cw as i32;
    let top = ay;
    let bottom = ay + CARD_HEIGHT as i32;

    let mut borders = Borders::empty();
    if left >= 0 {
        borders |= Borders::LEFT;
    }
    if right <= area.width as i32 {
        borders |= Borders::RIGHT;
    }
    if top >= 0 {
        borders |= Borders::TOP;
    }
    if bottom <= area.height as i32 {
        borders |= Borders::BOTTOM;
    }

    let x = (area.x as i32 + ax).max(area.x as i32) as u16;
    let y = (area.y as i32 + ay).max(area.y as i32) as u16;

    let width = (cw as i32 - (0 - ax).max(0))
        .max(0)
        .min((area.x as i32 + area.width as i32 - x as i32).max(0)) as u16;
    let height = (CARD_HEIGHT as i32 - (0 - ay).max(0))
        .max(0)
        .min((area.y as i32 + area.height as i32 - y as i32).max(0)) as u16;

    if width == 0 || height == 0 {
        return;
    }

    frame.render_widget(
        Block::bordered().borders(borders),
        Rect {
            x,
            y,
            width,
            height,
        },
    );

    // Compute inner from VIRTUAL position so text doesn't shift when clipped
    let inner_vx = ax + 1;
    let inner_vy = ay + 1;
    let inner_vw = cw as i32 - 2;
    let inner_vh = CARD_HEIGHT as i32 - 2;

    let inner_x = (area.x as i32 + inner_vx).max(area.x as i32) as u16;
    let inner_y = (area.y as i32 + inner_vy).max(area.y as i32) as u16;

    let inner_w = (inner_vw - (0 - inner_vx).max(0))
        .max(0)
        .min((area.x as i32 + area.width as i32 - inner_x as i32).max(0)) as u16;
    let inner_h = (inner_vh - (0 - inner_vy).max(0))
        .max(0)
        .min((area.y as i32 + area.height as i32 - inner_y as i32).max(0)) as u16;

    if inner_w == 0 || inner_h == 0 {
        return;
    }

    let top_won = series
        .winning_team_id
        .is_some_and(|id| series.top_seed_team.as_ref().is_some_and(|t| t.id == id));
    let bottom_won = series
        .winning_team_id
        .is_some_and(|id| series.bottom_seed_team.as_ref().is_some_and(|t| t.id == id));

    let top_team = series
        .top_seed_team
        .as_ref()
        .map(|t| t.common_name.default.clone())
        .unwrap_or_default();
    let bottom_team = series
        .bottom_seed_team
        .as_ref()
        .map(|t| t.common_name.default.clone())
        .unwrap_or_default();

    let (top_style, bottom_style) = match (top_won, bottom_won) {
        (true, _) => (
            Style::new().fg(COLOR_WIN).bold(),
            Style::new().fg(COLOR_LOSE),
        ),
        (_, true) => (
            Style::new().fg(COLOR_LOSE),
            Style::new().fg(COLOR_WIN).bold(),
        ),
        _ => (Style::default(), Style::default()),
    };

    let (top_seed_wins, bottom_seed_wins) =
        if series.top_seed_team.is_some() && series.bottom_seed_team.is_some() {
            (Some(series.top_seed_wins), Some(series.bottom_seed_wins))
        } else {
            (None, None)
        };

    // Seeding labels are only shown for round 1 (division/conference seeding).
    let (top_seed, bottom_seed) = if series.playoff_round == 1 {
        (
            seed_label(
                year,
                &series.series_letter,
                series.top_seed_rank,
                &series.top_seed_rank_abbrev,
            ),
            seed_label(
                year,
                &series.series_letter,
                series.bottom_seed_rank,
                &series.bottom_seed_rank_abbrev,
            ),
        )
    } else {
        (None, None)
    };

    let all_lines = vec![
        build_team_line(
            &top_team,
            top_seed.as_deref(),
            top_seed_wins,
            inner_w,
            top_style,
        ),
        Line::from(series.series_letter.clone())
            .centered()
            .style(Style::new().fg(Color::DarkGray)),
        build_team_line(
            &bottom_team,
            bottom_seed.as_deref(),
            bottom_seed_wins,
            inner_w,
            bottom_style,
        ),
    ];

    // Skip lines that are scrolled off the top so text stays anchored to its virtual position
    let lines_clipped_top = (0 - inner_vy).max(0) as usize;
    let visible_lines: Vec<Line> = all_lines.into_iter().skip(lines_clipped_top).collect();

    frame.render_widget(
        Paragraph::new(visible_lines),
        Rect {
            x: inner_x,
            y: inner_y,
            width: inner_w,
            height: inner_h,
        },
    );
}

fn build_team_line<'a>(
    abbrev: &str,
    seed: Option<&str>,
    wins: Option<u8>,
    width: u16,
    style: Style,
) -> Line<'a> {
    let wins_str = wins.map(|w| format!("{}", w)).unwrap_or_default();
    // Prefix the seed (e.g. "A1", "WC1", or "8") when available.
    let name_str = match seed {
        Some(s) if !s.is_empty() => format!("{s} {abbrev}"),
        _ => abbrev.to_string(),
    };
    let pad = (width as usize).saturating_sub(name_str.len() + wins_str.len());
    Line::from(vec![
        Span::styled(name_str, style),
        Span::raw(" ".repeat(pad)),
        Span::styled(wins_str, style.add_modifier(Modifier::BOLD)),
    ])
}

/// Division letters in the fixed order the NHL API returns round-1 series:
/// series A/B = Atlantic, C/D = Metropolitan, E/F = Central, G/H = Pacific.
const DIVISION_ORDER: [&str; 4] = ["A", "M", "C", "P"];

/// Compute the seeding label shown on a round-1 series card.
///
/// - 1994–2013: seeding was conference-wide (1 vs 8), so just show the numeric
///   `seed_rank` from the data.
/// - 2020: the COVID "bubble" playoffs used conference-wide reseeding (ranks up
///   to 12), so fall back to the numeric `seed_rank` like the pre-2014 era.
/// - Other 2014+ years: seeding is division-based. The API's `seed_rank_abbrev`
///   gives `D1`/`D2`/`D3` (division rank) or `WC1`/`WC2` (wildcard). Wildcards
///   are shown verbatim; for division seeds we substitute the specific division
///   letter, derived from the series letter's fixed position.
/// - Before 1994 (or an unrecognized series letter): no label.
fn seed_label(
    year: i32,
    series_letter: &str,
    seed_rank: u8,
    seed_rank_abbrev: &str,
) -> Option<String> {
    match year {
        1994..=2013 | 2020 => Some(seed_rank.to_string()),
        y if y >= 2014 => {
            if seed_rank_abbrev.starts_with("WC") {
                Some(seed_rank_abbrev.to_string())
            } else {
                // Expect "D1"/"D2"/"D3"; replace the leading "D" with the division letter.
                let division = division_from_series_letter(series_letter)?;
                let rank = seed_rank_abbrev.trim_start_matches(|c: char| c.is_ascii_alphabetic());
                Some(format!("{division}{rank}"))
            }
        }
        _ => None,
    }
}

/// Map a round-1 series letter (A–H) to its division letter using the fixed
/// order the API emits: A,B→Atlantic, C,D→Metro, E,F→Central, G,H→Pacific.
fn division_from_series_letter(series_letter: &str) -> Option<&'static str> {
    let idx = match series_letter {
        "A" | "B" => 0,
        "C" | "D" => 1,
        "E" | "F" => 2,
        "G" | "H" => 3,
        _ => return None,
    };
    Some(DIVISION_ORDER[idx])
}

fn draw_east_connectors(frame: &mut Frame, area: Rect, cw: u16, h_off: u16, v_off: u16) {
    draw_pair(frame, area, ["A", "B"], "I", cw, h_off, v_off, -1);
    draw_pair(frame, area, ["C", "D"], "J", cw, h_off, v_off, -1);
    draw_pair(frame, area, ["I", "J"], "M", cw, h_off, v_off, -1);
    draw_straight(frame, area, "M", "O", cw, h_off, v_off);
}

fn draw_west_connectors(frame: &mut Frame, area: Rect, cw: u16, h_off: u16, v_off: u16) {
    draw_pair(frame, area, ["E", "F"], "K", cw, h_off, v_off, 1);
    draw_pair(frame, area, ["G", "H"], "L", cw, h_off, v_off, 1);
    draw_pair(frame, area, ["K", "L"], "N", cw, h_off, v_off, 1);
    draw_straight(frame, area, "N", "O", cw, h_off, v_off);
}

/// Connect two series (vertically)
fn draw_pair(
    frame: &mut Frame,
    area: Rect,
    srcs: [&str; 2],
    dst: &str,
    card_w: u16,
    h_off: u16,
    v_off: u16,
    dir: i32, // +1 = right, -1 = left
) {
    let (Some((c0, r0)), Some((c1, r1)), Some((dc, _dr))) = (
        series_letter_to_position(srcs[0]),
        series_letter_to_position(srcs[1]),
        series_letter_to_position(dst),
    ) else {
        return;
    };

    let cw = card_w as i32;
    let gap = ROUND_HOR_GAP as i32;

    let src_x = c0 as i32 * (cw + gap) + if dir == 1 { cw } else { 0 };

    let dst_x = dc as i32 * (cw + gap) + if dir == 1 { 0 } else { cw };

    let mid_x = if dir == 1 {
        src_x + gap / 2
    } else {
        dst_x + gap / 2
    };

    let my0 = card_mid_y(c0, r0, card_w) as i32;
    let my1 = card_mid_y(c1, r1, card_w) as i32;
    let join_y = (my0 + my1) / 2;

    // horizontal from sources to mid
    let (start, end) = if dir == 1 {
        (src_x, mid_x)
    } else {
        (mid_x + 1, src_x)
    };

    for x in start..end {
        draw_char_from_virtual(frame, area, x as u16, my0 as u16, h_off, v_off, '─');
        draw_char_from_virtual(frame, area, x as u16, my1 as u16, h_off, v_off, '─');
    }

    // vertical join
    for y in (my0 + 1)..my1 {
        draw_char_from_virtual(frame, area, mid_x as u16, y as u16, h_off, v_off, '│');
    }

    // horizontal from mid to destination
    let (start, end) = if dir == 1 {
        (mid_x + 1, dst_x)
    } else {
        (dst_x, mid_x)
    };

    for x in start..end {
        draw_char_from_virtual(frame, area, x as u16, join_y as u16, h_off, v_off, '─');
    }
}

/// Draw a straight connector between series
fn draw_straight(
    frame: &mut Frame,
    area: Rect,
    src: &str,
    dst: &str,
    cw: u16,
    h_off: u16,
    v_off: u16,
) {
    let (Some((sc, sr)), Some((dc, _))) = (
        series_letter_to_position(src),
        series_letter_to_position(dst),
    ) else {
        return;
    };
    let src_x = sc as u16 * (cw + ROUND_HOR_GAP);
    let dst_x = dc as u16 * (cw + ROUND_HOR_GAP);
    // pick the correct edge of each card
    let src_edge = if src_x < dst_x {
        src_x + cw // going right
    } else {
        src_x // going left
    };
    let dst_edge = if src_x < dst_x {
        dst_x // entering from left
    } else {
        dst_x + cw // entering from right
    };
    let my = card_mid_y(sc, sr, cw);
    let (start, end) = if src_edge < dst_edge {
        (src_edge, dst_edge)
    } else {
        (dst_edge, src_edge)
    };
    for x in start..end {
        draw_char_from_virtual(frame, area, x, my, h_off, v_off, '─');
    }
}

/// Draw one character given the virtual position
fn draw_char_from_virtual(
    frame: &mut Frame,
    area: Rect,
    vx: u16,
    vy: u16,
    h_off: u16,
    v_off: u16,
    ch: char,
) {
    let ax = vx as i32 - h_off as i32;
    let ay = vy as i32 - v_off as i32;
    if ax < 0 || ay < 0 || ax >= area.width as i32 || ay >= area.height as i32 {
        return;
    }
    frame.render_widget(
        Line::from(Span::from(ch.to_string())),
        Rect {
            x: area.x + ax as u16,
            y: area.y + ay as u16,
            width: 1,
            height: 1,
        },
    );
}

fn render_scroll_indicators(frame: &mut Frame, area: Rect, playoff_bracket: &PlayoffsState) {
    let mid_x = area.x + area.width / 2;
    let mid_y = area.y + area.height / 2;
    if playoff_bracket.horizontal_scroll_offset > 0 {
        frame.render_widget(
            Line::from("◀"),
            Rect {
                x: area.x,
                y: mid_y,
                width: 1,
                height: 1,
            },
        );
    }
    if playoff_bracket.horizontal_scroll_offset < playoff_bracket.horizontal_max_scroll {
        frame.render_widget(
            Line::from("▶"),
            Rect {
                x: area.x + area.width - 1,
                y: mid_y,
                width: 1,
                height: 1,
            },
        );
    }
    if playoff_bracket.vertical_scroll_offset > 0 {
        frame.render_widget(
            Line::from("▲"),
            Rect {
                x: mid_x,
                y: area.y,
                width: 1,
                height: 1,
            },
        );
    }
    if playoff_bracket.vertical_scroll_offset < playoff_bracket.vertical_max_scroll {
        frame.render_widget(
            Line::from("▼"),
            Rect {
                x: mid_x,
                y: area.y + area.height - 1,
                width: 1,
                height: 1,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{division_from_series_letter, seed_label};

    #[test]
    fn division_from_letter_follows_fixed_order() {
        assert_eq!(division_from_series_letter("A"), Some("A"));
        assert_eq!(division_from_series_letter("B"), Some("A"));
        assert_eq!(division_from_series_letter("C"), Some("M"));
        assert_eq!(division_from_series_letter("D"), Some("M"));
        assert_eq!(division_from_series_letter("E"), Some("C"));
        assert_eq!(division_from_series_letter("F"), Some("C"));
        assert_eq!(division_from_series_letter("G"), Some("P"));
        assert_eq!(division_from_series_letter("H"), Some("P"));
        assert_eq!(division_from_series_letter("I"), None);
    }

    #[test]
    fn conference_era_shows_numeric_seed() {
        // 1994..=2013: use the numeric seed rank, ignore the abbrev.
        assert_eq!(seed_label(2010, "A", 1, "D1").as_deref(), Some("1"));
        assert_eq!(seed_label(2010, "A", 8, "C8").as_deref(), Some("8"));
        assert_eq!(seed_label(2013, "D", 5, "C5").as_deref(), Some("5"));
    }

    #[test]
    fn covid_2020_falls_back_to_numeric_seed() {
        // 2020 bubble used conference-wide reseeding (ranks up to 12).
        assert_eq!(seed_label(2020, "A", 1, "C4").as_deref(), Some("1"));
        assert_eq!(seed_label(2020, "A", 8, "C12").as_deref(), Some("8"));
        assert_eq!(seed_label(2020, "D", 4, "C1").as_deref(), Some("4"));
    }

    #[test]
    fn division_era_uses_division_letter_and_wildcard() {
        // 2014+: division seeds get the specific division letter.
        assert_eq!(seed_label(2024, "A", 1, "D1").as_deref(), Some("A1"));
        assert_eq!(seed_label(2024, "B", 2, "D2").as_deref(), Some("A2"));
        assert_eq!(seed_label(2024, "B", 3, "D3").as_deref(), Some("A3"));
        assert_eq!(seed_label(2024, "D", 2, "D2").as_deref(), Some("M2"));
        assert_eq!(seed_label(2024, "F", 3, "D3").as_deref(), Some("C3"));
        assert_eq!(seed_label(2024, "H", 2, "D2").as_deref(), Some("P2"));
        // Wildcards are shown verbatim regardless of series letter.
        assert_eq!(seed_label(2024, "A", 4, "WC1").as_deref(), Some("WC1"));
        assert_eq!(seed_label(2024, "C", 4, "WC2").as_deref(), Some("WC2"));
    }

    #[test]
    fn unsupported_years_have_no_label() {
        assert_eq!(seed_label(1993, "A", 1, "D1"), None);
        assert_eq!(seed_label(1980, "A", 1, ""), None);
    }
}
