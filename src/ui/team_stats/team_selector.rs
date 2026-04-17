use crate::state::team_stats::team_picker::TeamPickerState;
use crate::ui::{input_popup::InputPopup, render::BORDER_FOCUSED_COLOR};
use ratatui::{prelude::*, style::Color};

pub struct TeamSelectorWidget {}

impl StatefulWidget for TeamSelectorWidget {
    type State = TeamPickerState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let border_color = if state.is_valid {
            BORDER_FOCUSED_COLOR
        } else {
            Color::Red
        };
        InputPopup {
            title: "Enter a team abbreviation",
            instructions: "Press Enter to submit or Esc to cancel",
            input_text: &state.text,
            border_color,
            info: None,
        }
        .render(area, buf);
    }
}
