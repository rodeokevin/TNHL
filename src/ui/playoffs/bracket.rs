use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;
use crate::models::playoffs::bracket::Series;
use crate::state::app_state::PaneFocus;
use crate::state::playoff_bracket::BracketState;
use crate::ui::render::{BORDER_FOCUSED_COLOR, BORDER_UNFOCUSED_COLOR};

const CARD_WIDTH: u16 = 14;
const CARD_HEIGHT: u16 = 5;
// Horizontal gap length between rounds
const ROUND_HOR_GAP: u16 = 6;

const COLOR_WIN: Color = Color::Green;
const COLOR_LOSE: Color = Color::DarkGray;

pub fn render_playoffs(frame: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.state.focus == PaneFocus::Content;
    let border_style = if focused {
        Style::default()
            .fg(BORDER_FOCUSED_COLOR)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(BORDER_UNFOCUSED_COLOR)
    };
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(format!(
            " {} Stanley Cup Playoffs ",
            app.state.date_state.year
        ));

    let inner = outer_block.inner(area);
    // Pass visible rows/columns and max scroll to playoff_bracket state
    app.state.playoff_bracket.visible_columns = inner.width as usize;
    app.state.playoff_bracket.visible_rows = inner.height as usize;

    app.state.playoff_bracket.horizontal_max_scroll =
        canvas_width().saturating_sub(inner.width) as usize;
    app.state.playoff_bracket.vertical_max_scroll =
        canvas_height().saturating_sub(inner.height) as usize;

    frame.render_widget(outer_block, area);

    if let Some(playoff_bracket) = &app.state.playoff_bracket.playoff_bracket_data {
        let h_off = app.state.playoff_bracket.horizontal_scroll_offset as u16;
        let v_off = app.state.playoff_bracket.vertical_scroll_offset as u16;

        let bracket_area = Rect {
            x: inner.x + 1,
            y: inner.y + 1,
            width: inner.width.saturating_sub(2),
            height: inner.height.saturating_sub(2),
        };
        render_bracket(frame, bracket_area, &playoff_bracket.series, h_off, v_off);
        render_scroll_indicators(frame, inner, &app.state.playoff_bracket);
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

fn canvas_width() -> u16 {
    7 * CARD_WIDTH + 6 * ROUND_HOR_GAP
}
fn canvas_height() -> u16 {
    1 + 4 * CARD_HEIGHT + 3 * 2
} // 1 label row + 4 cards + 3×2 gaps

fn r1_y(row: usize) -> u16 {
    let gap = 1;
    1 + row as u16 * (CARD_HEIGHT + gap)
}
fn midpoint(a: u16, b: u16) -> u16 {
    (a + b) / 2
}

// Compute the card position based on R1 cards
fn card_virtual_pos(col: usize, row: usize) -> (u16, u16) {
    let x = col as u16 * (CARD_WIDTH + ROUND_HOR_GAP);

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

fn card_mid_y(col: usize, row: usize) -> u16 {
    card_virtual_pos(col, row).1 + CARD_HEIGHT / 2
}

fn render_bracket(frame: &mut Frame, area: Rect, series_list: &[Series], h_off: u16, v_off: u16) {
    // Column labels
    for (col, label) in COLUMN_LABELS.iter().enumerate() {
        let vx = col as u16 * (CARD_WIDTH + ROUND_HOR_GAP);
        draw_round_label(frame, area, vx, 0, CARD_WIDTH, label, h_off, v_off);
    }

    // Series cards — look up each series' column/row from its letter
    for series in series_list {
        let Some((col, row)) = series_letter_to_position(&series.series_letter) else {
            continue;
        };
        let (vx, vy) = card_virtual_pos(col, row);
        render_series_card(frame, area, series, vx, vy, h_off, v_off);
    }

    // Connectors
    draw_east_connectors(frame, area, h_off, v_off);
    draw_west_connectors(frame, area, h_off, v_off);
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
        Line::from(label.to_string()).alignment(Alignment::Center),
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
    vx: u16,
    vy: u16,
    h_off: u16,
    v_off: u16,
) {
    // compute the actual coordinates
    let ax = vx as i32 - h_off as i32;
    let ay = vy as i32 - v_off as i32;
    // Early return if card not visible
    if ax + CARD_WIDTH as i32 <= 0
        || ax >= area.width as i32
        || ay + CARD_HEIGHT as i32 <= 0
        || ay >= area.height as i32
    {
        return;
    }

    let left = ax;
    let right = ax + CARD_WIDTH as i32;
    let top = ay;
    let bottom = ay + CARD_HEIGHT as i32;

    // Determine visible borders
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

    // Compute clipped rect
    let x = (area.x as i32 + ax).max(area.x as i32) as u16;
    let y = (area.y as i32 + ay).max(area.y as i32) as u16;

    let width = (CARD_WIDTH as i32 - (0 - ax).max(0))
        .max(0)
        .min((area.x as i32 + area.width as i32 - x as i32).max(0)) as u16;

    let height = (CARD_HEIGHT as i32 - (0 - ay).max(0))
        .max(0)
        .min((area.y as i32 + area.height as i32 - y as i32).max(0)) as u16;

    if width == 0 || height == 0 {
        return;
    }

    let rect = Rect {
        x,
        y,
        width,
        height,
    };
    frame.render_widget(Block::bordered().borders(borders), rect);

    // Inner text
    let inner_w = rect.width.saturating_sub(2);
    let inner_h = rect.height.saturating_sub(2);
    if inner_w == 0 || inner_h == 0 {
        return;
    }

    let inner = Rect {
        x: rect.x + 1,
        y: rect.y + 1,
        width: inner_w,
        height: inner_h,
    };

    let top_won = series
        .winning_team_id
        .is_some_and(|id| series.top_seed_team.as_ref().is_some_and(|t| t.id == id));
    let bottom_won = series
        .winning_team_id
        .is_some_and(|id| series.bottom_seed_team.as_ref().is_some_and(|t| t.id == id));

    let top_abbrev = series
        .top_seed_team
        .as_ref()
        .map(|t| t.abbrev.to_string())
        .unwrap_or_default();
    let bottom_abbrev = series
        .bottom_seed_team
        .as_ref()
        .map(|t| t.abbrev.to_string())
        .unwrap_or_default();

    let bold = Modifier::BOLD;
    let (top_style, bottom_style) = match (top_won, bottom_won) {
        (true, _) => (
            Style::default().fg(COLOR_WIN).add_modifier(bold),
            Style::default().fg(COLOR_LOSE),
        ),
        (_, true) => (
            Style::default().fg(COLOR_LOSE),
            Style::default().fg(COLOR_WIN).add_modifier(bold),
        ),
        _ => (Style::default(), Style::default()),
    };

    let (top_seed_wins, bottom_seed_wins) =
        if series.top_seed_team.is_some() && series.bottom_seed_team.is_some() {
            (Some(series.top_seed_wins), Some(series.bottom_seed_wins))
        } else {
            (None, None)
        };

    let lines = vec![
        build_team_line(&top_abbrev, top_seed_wins, inner_w, top_style),
        Line::from(series.series_letter.clone())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray)),
        build_team_line(&bottom_abbrev, bottom_seed_wins, inner_w, bottom_style),
    ];

    frame.render_widget(Paragraph::new(lines), inner);
}

fn build_team_line<'a>(abbrev: &str, wins: Option<u8>, width: u16, style: Style) -> Line<'a> {
    let wins_str = wins.map(|w| format!("{}", w)).unwrap_or_default();
    let abbrev_str = abbrev.to_string();
    let pad = (width as usize).saturating_sub(abbrev_str.len() + wins_str.len());
    Line::from(vec![
        Span::styled(abbrev_str, style),
        Span::raw(" ".repeat(pad)),
        Span::styled(wins_str, style.add_modifier(Modifier::BOLD)),
    ])
}

fn draw_east_connectors(frame: &mut Frame, area: Rect, h_off: u16, v_off: u16) {
    draw_pair(frame, area, ["A", "B"], "I", h_off, v_off, -1);
    draw_pair(frame, area, ["C", "D"], "J", h_off, v_off, -1);
    draw_pair(frame, area, ["I", "J"], "M", h_off, v_off, -1);
    draw_straight(frame, area, "M", "O", h_off, v_off);
}

fn draw_west_connectors(frame: &mut Frame, area: Rect, h_off: u16, v_off: u16) {
    draw_pair(frame, area, ["E", "F"], "K", h_off, v_off, 1);
    draw_pair(frame, area, ["G", "H"], "L", h_off, v_off, 1);
    draw_pair(frame, area, ["K", "L"], "N", h_off, v_off, 1);
    draw_straight(frame, area, "N", "O", h_off, v_off);
}

/// Connect two series (vertically)
fn draw_pair(
    frame: &mut Frame,
    area: Rect,
    srcs: [&str; 2],
    dst: &str,
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

    let cw = CARD_WIDTH as i32;
    let gap = ROUND_HOR_GAP as i32;

    let src_x = c0 as i32 * (cw + gap) + if dir == 1 { cw } else { 0 };

    let dst_x = dc as i32 * (cw + gap) + if dir == 1 { 0 } else { cw };

    let mid_x = if dir == 1 {
        src_x + gap / 2
    } else {
        dst_x + gap / 2
    };

    let my0 = card_mid_y(c0, r0) as i32;
    let my1 = card_mid_y(c1, r1) as i32;
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
fn draw_straight(frame: &mut Frame, area: Rect, src: &str, dst: &str, h_off: u16, v_off: u16) {
    let (Some((sc, sr)), Some((dc, _))) = (
        series_letter_to_position(src),
        series_letter_to_position(dst),
    ) else {
        return;
    };
    let src_x = sc as u16 * (CARD_WIDTH + ROUND_HOR_GAP);
    let dst_x = dc as u16 * (CARD_WIDTH + ROUND_HOR_GAP);
    // pick the correct edge of each card
    let src_edge = if src_x < dst_x {
        src_x + CARD_WIDTH // going right
    } else {
        src_x // going left
    };
    let dst_edge = if src_x < dst_x {
        dst_x // entering from left
    } else {
        dst_x + CARD_WIDTH // entering from right
    };
    let my = card_mid_y(sc, sr);
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

fn render_scroll_indicators(frame: &mut Frame, area: Rect, playoff_bracket: &BracketState) {
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
