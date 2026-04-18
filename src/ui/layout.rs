use crate::ui::input_popup;
use ratatui::layout::Rect;

pub struct LayoutAreas {}

impl LayoutAreas {
    /// Create a centered rectangle of 4 height and 42% width for any picker popup.
    pub fn create_picker_rect(area: Rect) -> Rect {
        input_popup::create_popup(area, 4, 42)
    }
}
