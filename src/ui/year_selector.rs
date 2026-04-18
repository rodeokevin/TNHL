use crate::state::date_state::DateState;
use crate::ui::{input_popup::InputPopup, render::BORDER_FOCUSED_COLOR};
use ratatui::{prelude::*, style::Color};

pub struct YearSelectorWidget {}

impl StatefulWidget for YearSelectorWidget {
    type State = DateState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let border_color = if state.is_valid {
            BORDER_FOCUSED_COLOR
        } else {
            Color::Red
        };
        InputPopup {
            title: "Enter a year (YYYY) or use arrow keys",
            instructions: "Press Enter to submit or Esc to cancel",
            input_text: &state.text,
            border_color,
            info: None,
            bottom_text: Some("Hint: enter 2026 for 2025-2026 season"),
        }
        .render(area, buf);
    }
}
