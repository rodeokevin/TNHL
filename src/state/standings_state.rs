use ratatui::widgets::TableState;

const LEAGUE_NUM_TEAMS: usize = 32;
const CONFERENCE_NUM_TEAMS: usize = 16;
const DIVISION_NUM_TEAMS: usize = 8;
const WILDCARD_NUM_TEAMS: usize = 16 + 3; // + 3 for the division/conference rows

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
    pub focus: StandingsFocus,

    pub selected_conference: ConferenceFocus,
    pub eastern_table_state: TableState,
    pub western_table_state: TableState,

    pub selected_division: DivisionFocus,
    pub atlantic_table_state: TableState,
    pub metropolitan_table_state: TableState,
    pub central_table_state: TableState,
    pub pacific_table_state: TableState,

    pub selected_wildcard: ConferenceFocus,
    pub eastern_wildcard_table_state: TableState,
    pub western_wildcard_table_state: TableState,

    pub league_table_state: TableState,
}

impl Default for StandingsState {
    fn default() -> Self {
        fn table() -> TableState {
            let mut t = TableState::default();
            t.select(Some(0));
            t
        }

        Self {
            focus: StandingsFocus::default(),

            selected_conference: ConferenceFocus::default(),
            eastern_table_state: table(),
            western_table_state: table(),

            selected_division: DivisionFocus::default(),
            atlantic_table_state: table(),
            metropolitan_table_state: table(),
            central_table_state: table(),
            pacific_table_state: table(),

            selected_wildcard: ConferenceFocus::default(),
            eastern_wildcard_table_state: table(),
            western_wildcard_table_state: table(),

            league_table_state: table(),
        }
    }
}

impl StandingsState {
    pub fn current_table_state_mut(&mut self) -> (&mut TableState, usize) {
        match self.focus {
            StandingsFocus::League => (&mut self.league_table_state, LEAGUE_NUM_TEAMS),
            StandingsFocus::Conference => match self.selected_conference {
                ConferenceFocus::Eastern => (&mut self.eastern_table_state, CONFERENCE_NUM_TEAMS),
                ConferenceFocus::Western => (&mut self.western_table_state, CONFERENCE_NUM_TEAMS),
            },
            StandingsFocus::Division => match self.selected_division {
                DivisionFocus::Atlantic => (&mut self.atlantic_table_state, DIVISION_NUM_TEAMS),
                DivisionFocus::Metropolitan => {
                    (&mut self.metropolitan_table_state, DIVISION_NUM_TEAMS)
                }
                DivisionFocus::Central => (&mut self.central_table_state, DIVISION_NUM_TEAMS),
                DivisionFocus::Pacific => (&mut self.pacific_table_state, DIVISION_NUM_TEAMS),
            },
            StandingsFocus::WildCard => match self.selected_wildcard {
                ConferenceFocus::Eastern => {
                    (&mut self.eastern_wildcard_table_state, WILDCARD_NUM_TEAMS)
                }
                ConferenceFocus::Western => {
                    (&mut self.western_wildcard_table_state, WILDCARD_NUM_TEAMS)
                }
            },
        }
    }

    pub fn move_selection(&mut self, delta: i32) {
        let (table, len) = self.current_table_state_mut();
        let current = table.selected().unwrap_or(0);
        let new = current as i32 + delta;
        let next = if new < 0 || new >= len as i32 {
            current
        } else {
            new as usize
        };
        table.select(Some(next));
    }

    // Next/PrevStandings
    pub fn shift_standings_type(&mut self, next: bool) {
        self.focus = if next {
            self.focus.next()
        } else {
            self.focus.prev()
        };
    }

    // Cycle between a standings type
    pub fn cycle_focus(&mut self, next: bool) {
        match self.focus {
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
}
