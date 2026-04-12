use ratatui::widgets::TableState;

use crate::{models::standings::StandingsResponse};

const LEAGUE_NUM_TEAMS: usize = 32;
const CONFERENCE_NUM_TEAMS: usize = 16;
const DIVISION_NUM_TEAMS: usize = 8;
const WILDCARD_NUM_TEAMS: usize = 16 + 3; // + 3 for the division/conference name rows

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum StandingsFocus {
    #[default]
    WildCard,
    Division,
    Conference,
    League,
}

impl StandingsFocus {
    pub fn next(self) -> Self {
        match self {
            StandingsFocus::WildCard => StandingsFocus::Division,
            StandingsFocus::Division => StandingsFocus::Conference,
            StandingsFocus::Conference => StandingsFocus::League,
            StandingsFocus::League => StandingsFocus::League,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            StandingsFocus::WildCard => StandingsFocus::WildCard,
            StandingsFocus::Division => StandingsFocus::WildCard,
            StandingsFocus::Conference => StandingsFocus::Division,
            StandingsFocus::League => StandingsFocus::Conference,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ConferenceFocus {
    #[default]
    Eastern,
    Western,
}

impl ConferenceFocus {
    pub fn toggle(self) -> Self {
        match self {
            ConferenceFocus::Eastern => ConferenceFocus::Western,
            ConferenceFocus::Western => ConferenceFocus::Eastern,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DivisionFocus {
    #[default]
    Atlantic,
    Metropolitan,
    Central,
    Pacific,
}

impl DivisionFocus {
    pub fn next(self) -> Self {
        match self {
            DivisionFocus::Atlantic => DivisionFocus::Metropolitan,
            DivisionFocus::Metropolitan => DivisionFocus::Central,
            DivisionFocus::Central => DivisionFocus::Pacific,
            DivisionFocus::Pacific => DivisionFocus::Atlantic,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            DivisionFocus::Atlantic => DivisionFocus::Pacific,
            DivisionFocus::Metropolitan => DivisionFocus::Atlantic,
            DivisionFocus::Central => DivisionFocus::Metropolitan,
            DivisionFocus::Pacific => DivisionFocus::Central,
        }
    }
}

pub struct StandingsState {
    pub standings_data: Option<StandingsResponse>,
    pub table_state: TableState,
    /// Number of visible rows in the table, updated during render
    pub visible_rows: usize,

    pub selected_standings: StandingsFocus,
    pub selected_conference: ConferenceFocus,
    pub selected_division: DivisionFocus,
    pub selected_wildcard: ConferenceFocus,
}

impl Default for StandingsState {
    fn default() -> Self {
        fn table() -> TableState {
            let mut t = TableState::default();
            t.select(Some(0));
            t
        }

        Self {
            standings_data: None,
            table_state: table(),
            visible_rows: 0,

            selected_standings: StandingsFocus::default(),
            selected_conference: ConferenceFocus::default(),
            selected_division: DivisionFocus::default(),
            selected_wildcard: ConferenceFocus::default(),
        }
    }
}

impl StandingsState {
    /// Return the length of the active standings table
    pub fn current_table_len(&mut self) -> usize {
        match self.selected_standings {
            StandingsFocus::League => LEAGUE_NUM_TEAMS,
            StandingsFocus::Conference => CONFERENCE_NUM_TEAMS,
            StandingsFocus::Division => DIVISION_NUM_TEAMS,
            StandingsFocus::WildCard => WILDCARD_NUM_TEAMS,
        }
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
    /// Next/PrevStandings
    pub fn shift_standings_type(&mut self, next: bool) {
        self.selected_standings = if next {
            self.selected_standings.next()
        } else {
            self.selected_standings.prev()
        };
    }
    /// Cycle within a selected standings (e.g. change division in division standings)
    pub fn cycle_display(&mut self, next: bool) {
        match self.selected_standings {
            StandingsFocus::Conference => {
                self.selected_conference = self.selected_conference.toggle()
            }
            StandingsFocus::Division => {
                self.selected_division = if next {
                    self.selected_division.next()
                } else {
                    self.selected_division.prev()
                };
            }
            StandingsFocus::WildCard => self.selected_wildcard = self.selected_wildcard.toggle(),
            StandingsFocus::League => {}
        }
    }
    /// Reset standings to default state
    pub fn reset_state(&mut self) {
        self.reset_table_state();

        self.selected_standings = StandingsFocus::default();
        self.selected_conference = ConferenceFocus::default();
        self.selected_division = DivisionFocus::default();
        self.selected_wildcard = ConferenceFocus::default();
    }
    /// Reset selected row in table
    pub fn reset_table_state(&mut self) {
        self.table_state.select(Some(0));
    }
}
