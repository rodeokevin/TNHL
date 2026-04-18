use crate::models::playoffs::bracket::BracketResponse;

pub struct BracketState {
    pub playoff_bracket_data: Option<BracketResponse>,
    /// Visible rows and columns updated at render
    pub visible_rows: usize,
    pub visible_columns: usize,
    // The letter representing the series. If None, the ui displays the bracket
    pub selected_series: Option<String>,
    // Todo: selected series state
    pub horizontal_scroll_offset: usize,
    /// Max horizontal scroll updated at render
    pub horizontal_max_scroll: usize,
    pub vertical_scroll_offset: usize,
    /// Max vertical scroll updated at render
    pub vertical_max_scroll: usize,
}

impl Default for BracketState {
    fn default() -> Self {
        Self {
            playoff_bracket_data: None,
            visible_rows: 0,
            visible_columns: 0,
            selected_series: None,

            horizontal_scroll_offset: 0,
            horizontal_max_scroll: 0,
            vertical_scroll_offset: 0,
            vertical_max_scroll: 0,
        }
    }
}

impl BracketState {
    pub fn reset_scoring_scroll(&mut self) {
        self.horizontal_scroll_offset = 0;
        self.horizontal_max_scroll = 0;
        self.vertical_scroll_offset = 0;
        self.vertical_max_scroll = 0;
    }
    /// Reset all state to default
    pub fn reset_state(&mut self) {
        self.reset_scoring_scroll();
        self.selected_series = None;
    }
    /// Page up for bracket page
    pub fn bracket_page_up(&mut self) {
        if self.visible_rows != 0 {
            self.vertical_scroll_offset = self
                .vertical_scroll_offset
                .saturating_sub(self.visible_rows);
        }
    }
    /// Page down bracket
    pub fn bracket_page_down(&mut self) {
        if self.visible_rows != 0 {
            self.vertical_scroll_offset =
                (self.vertical_scroll_offset + self.visible_rows).min(self.vertical_max_scroll);
        }
    }
    /// Page left for bracket page
    pub fn bracket_page_left(&mut self) {
        if self.visible_columns != 0 {
            self.horizontal_scroll_offset = self
                .horizontal_scroll_offset
                .saturating_sub(self.visible_columns);
        }
    }
    /// Page right bracket
    pub fn bracket_page_right(&mut self) {
        if self.visible_rows == 0 {
            return;
        }
        self.horizontal_scroll_offset =
            (self.horizontal_scroll_offset + self.visible_columns).min(self.horizontal_max_scroll);
    }
}
