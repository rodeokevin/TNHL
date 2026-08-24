use crate::models::playoffs::bracket::BracketResponse;
use crate::models::playoffs::series::SeriesResponse;

/// Cells moved per arrow-key scroll in the bracket.
const BRACKET_SCROLL_STEP: usize = 2;
/// Cells moved per page (Shift + arrow) in the bracket. Fixed rather than a
/// full screen so paging the large canvas isn't a huge jump.
const BRACKET_PAGE_STEP: usize = 10;

pub struct PlayoffsState {
    pub focus: PlayoffsFocus,
    pub bracket_data: Option<BracketResponse>,
    /// Visible rows and columns updated at render
    pub visible_rows: usize,
    pub visible_columns: usize,
    // The letter representing the series. If None, the ui displays the bracket
    pub selected_series: Option<char>,
    pub series_data: Option<SeriesResponse>,
    pub horizontal_scroll_offset: usize,
    /// Max horizontal scroll updated at render
    pub horizontal_max_scroll: usize,
    pub vertical_scroll_offset: usize,
    /// Max vertical scroll updated at render
    pub vertical_max_scroll: usize,
}

impl Default for PlayoffsState {
    fn default() -> Self {
        Self {
            focus: PlayoffsFocus::default(),
            bracket_data: None,
            visible_rows: 0,
            visible_columns: 0,

            selected_series: None,
            series_data: None,

            horizontal_scroll_offset: 0,
            horizontal_max_scroll: 0,
            vertical_scroll_offset: 0,
            vertical_max_scroll: 0,
        }
    }
}

impl PlayoffsState {
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
        self.series_data = None;
        self.focus = PlayoffsFocus::default();
    }
    /// Scroll the bracket by a fixed step.
    pub fn scroll_up(&mut self) {
        self.vertical_scroll_offset = self
            .vertical_scroll_offset
            .saturating_sub(BRACKET_SCROLL_STEP);
    }
    pub fn scroll_down(&mut self) {
        self.vertical_scroll_offset =
            (self.vertical_scroll_offset + BRACKET_SCROLL_STEP).min(self.vertical_max_scroll);
    }
    pub fn scroll_left(&mut self) {
        if self.focus == PlayoffsFocus::Bracket {
            self.horizontal_scroll_offset = self
                .horizontal_scroll_offset
                .saturating_sub(BRACKET_SCROLL_STEP);
        }
    }
    pub fn scroll_right(&mut self) {
        if self.focus == PlayoffsFocus::Bracket {
            self.horizontal_scroll_offset = (self.horizontal_scroll_offset + BRACKET_SCROLL_STEP)
                .min(self.horizontal_max_scroll);
        }
    }
    /// Page up by a fixed step.
    pub fn page_up(&mut self) {
        self.vertical_scroll_offset = self
            .vertical_scroll_offset
            .saturating_sub(BRACKET_PAGE_STEP);
    }
    /// Page down by a fixed step.
    pub fn page_down(&mut self) {
        self.vertical_scroll_offset =
            (self.vertical_scroll_offset + BRACKET_PAGE_STEP).min(self.vertical_max_scroll);
    }
    /// Page left by a fixed step.
    pub fn page_left(&mut self) {
        if self.focus == PlayoffsFocus::Bracket {
            self.horizontal_scroll_offset = self
                .horizontal_scroll_offset
                .saturating_sub(BRACKET_PAGE_STEP);
        }
    }
    /// Page right by a fixed step.
    pub fn page_right(&mut self) {
        if self.focus == PlayoffsFocus::Bracket {
            self.horizontal_scroll_offset =
                (self.horizontal_scroll_offset + BRACKET_PAGE_STEP).min(self.horizontal_max_scroll);
        }
    }
    /// Try selecting a series using a letter if it exists. Return true if succeeded
    pub fn try_select_series(&mut self, letter: char) -> bool {
        let letter = letter.to_ascii_uppercase();

        if let Some(data) = &self.bracket_data {
            if data
                .series
                .iter()
                .any(|s| s.series_letter == letter.to_string())
            {
                self.selected_series = Some(letter);
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PlayoffsFocus {
    #[default]
    Bracket,
    Series,
}
