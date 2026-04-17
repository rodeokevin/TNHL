use ratatui::widgets::TableState;

use crate::models::team_stats::TeamStatsResponse;
use crate::state::team_stats::team_picker::TeamPickerState;

pub struct TeamStatsState {
    pub team_stats_data: Option<TeamStatsResponse>,
    pub table_state: TableState,
    /// Number of visible rows in the table, updated during render
    pub visible_rows: usize,
    pub show_skaters: bool, // true: skaters (fowards + defense), false: goalies
    pub team_picker: TeamPickerState,
}

impl Default for TeamStatsState {
    fn default() -> Self {
        fn table() -> TableState {
            let mut t = TableState::default();
            t.select(Some(0));
            t
        }

        Self {
            team_stats_data: None,
            table_state: table(),
            visible_rows: 0,
            show_skaters: true,
            team_picker: TeamPickerState::default(),
        }
    }
}

impl TeamStatsState {
    /// Return the length of the table
    pub fn current_table_len(&self) -> usize {
        self.team_stats_data
            .as_ref()
            .map(|data| {
                if self.show_skaters {
                    data.skaters.len()
                } else {
                    data.goalies.len()
                }
            })
            .unwrap_or(0)
    }
    /// Select a new row in the standings table
    pub fn move_selection(&mut self, delta: i32) {
        let len = self.current_table_len();
        let current = self.table_state.selected().unwrap_or(0);
        let new = current as i32 + delta;
        let next = new.clamp(0, (len - 1) as i32) as usize;
        self.table_state.select(Some(next));
    }
    pub fn page_up(&mut self) {
        if self.visible_rows == 0 {
            return;
        }
        // The first visible row becomes the last visible row
        let offset = self.table_state.offset();
        let new_offset = offset.saturating_sub(self.visible_rows - 1);
        *self.table_state.offset_mut() = new_offset;
        self.table_state.select(Some(new_offset));
    }
    pub fn page_down(&mut self) {
        if self.visible_rows == 0 {
            return;
        }
        // The last visible row becomes the first visible row
        // But if last visible row is the last row in the table, simply select it without changing the offset
        let len = self.current_table_len();
        let offset = self.table_state.offset();
        let last_visible = if offset + self.visible_rows - 1 >= len - 1 {
            len - 1
        } else {
            *self.table_state.offset_mut() = (offset + self.visible_rows - 1).min(len - 1);
            (offset + self.visible_rows - 1).min(len - 1)
        };
        self.table_state.select(Some(last_visible));
    }
    /// Reset standings to default state
    pub fn reset_state(&mut self) {
        self.reset_table_state();

        self.show_skaters = true;
    }
    /// Reset selected row in table
    pub fn reset_table_state(&mut self) {
        self.table_state.select(Some(0));
    }
}
