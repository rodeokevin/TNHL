use crate::ui::input_popup;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use std::rc::Rc;

/// Height (in rows) of the tab/header band at the top of the tabbed pages
/// (Games, Standings). Includes the 1-row border on each side of the tab text.
pub const TAB_BAND_HEIGHT: u16 = 3;

pub struct LayoutAreas {}

impl LayoutAreas {
    /// Create a centered rectangle of 4 height and 42% width for any picker popup.
    pub fn create_picker_rect(area: Rect) -> Rect {
        input_popup::create_popup(area, 4, 42)
    }
}

/// Split an area into columns using the given horizontal constraints.
pub fn split_area_horizontal(area: Rect, constraints: impl Into<Vec<Constraint>>) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints.into())
        .split(area)
        .to_vec()
}

/// Split an area into rows using the given vertical constraints.
pub fn split_area_vertical(area: Rect, constraints: impl Into<Vec<Constraint>>) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints.into())
        .split(area)
}

/// Split a page area into the standard `[tab band, content]` layout shared by
/// the tabbed pages (Games, Standings): a fixed-height header band on top and
/// the remaining space for content below.
pub fn tabs_and_content(area: Rect) -> Rc<[Rect]> {
    split_area_vertical(
        area,
        [Constraint::Length(TAB_BAND_HEIGHT), Constraint::Min(1)],
    )
}
