use crate::ui::input_popup;
use ratatui::layout::Rect;

pub struct LayoutAreas {}

impl LayoutAreas {
    /// Create a centered rectangle of 4 height and 42% width for the date picker.
    pub fn create_date_picker(area: Rect) -> Rect {
        input_popup::create_popup(area, 4, 42)
    }
    /// Create a centered rectangle of 4 height and 42% width for the team picker.
    pub fn create_team_picker(area: Rect) -> Rect {
        input_popup::create_popup(area, 4, 42)
    }
}
