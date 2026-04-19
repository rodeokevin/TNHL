use crate::models::team_stats::TeamStatsResponse;
use crate::state::{
    app_state::{table_page_down, table_page_up},
    team_stats::team_picker::TeamPickerState,
};
use ratatui::widgets::TableState;

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
    /// Move rows
    pub fn row_up(&mut self) {
        self.table_state.scroll_up_by(1);
    }
    pub fn row_down(&mut self) {
        self.table_state.scroll_down_by(1);
    }
    pub fn page_up(&mut self) {
        table_page_up(self.visible_rows, &mut self.table_state);
    }
    pub fn page_down(&mut self) {
        table_page_down(
            self.visible_rows,
            self.current_table_len(),
            &mut self.table_state,
        );
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
