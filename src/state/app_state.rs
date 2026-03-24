use std::fmt::Debug;
use std::thread::current;

use crossterm::event::{KeyCode, KeyCode::Char, KeyEvent, KeyModifiers};

use crate::input::{Action, map_key};
use crate::models::games::GamesResponse;
use crate::models::standings::StandingsResponse;
use crate::sources::AppEvent;
use crate::state::date_input::DateInput;
use crate::state::date_selector::DateSelector;
use crate::state::standings_state::StandingsState;

/// Which pane currently has keyboard focus.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum PaneFocus {
    #[default]
    Menu,
    Content,
    DatePicker,
}

impl PaneFocus {
    pub fn switch(self) -> Self {
        match self {
            PaneFocus::Menu => PaneFocus::Content,
            PaneFocus::Content => PaneFocus::Menu,
            PaneFocus::DatePicker => PaneFocus::DatePicker,
        }
    }
}

/// Which menu item is currently selected.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MenuFocus {
    #[default]
    Games,
    Standings,
    Teams,
}

impl MenuFocus {
    pub fn next(self) -> Self {
        match self {
            MenuFocus::Games => MenuFocus::Standings,
            MenuFocus::Standings => MenuFocus::Teams,
            MenuFocus::Teams => MenuFocus::Teams,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            MenuFocus::Standings => MenuFocus::Games,
            MenuFocus::Teams => MenuFocus::Standings,
            MenuFocus::Games => MenuFocus::Games,
        }
    }
    pub fn index(&self) -> usize {
        match self {
            MenuFocus::Games => 0,
            MenuFocus::Standings => 1,
            MenuFocus::Teams => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Normal,
    EditingDate,
}

pub struct AppState {
    pub date_input: DateInput,
    pub date_selector: DateSelector,

    pub selected_menu: MenuFocus,
    pub standings: StandingsState,
    pub league_data: Option<StandingsResponse>,

    pub games_data: Option<GamesResponse>,
    pub selected_game_index: usize,
    pub sweeping_status_offset: usize, // For the --- under the time remaining
    pub scoring_scroll_offset: usize,
    pub max_scoring_scroll: usize,

    pub focus: PaneFocus,
    pub previous_focus: PaneFocus,
    pub should_quit: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            date_input: DateInput::default(),
            date_selector: DateSelector::default(),

            selected_menu: MenuFocus::default(),
            standings: StandingsState::default(),
            league_data: None,

            games_data: None,
            selected_game_index: 0,
            sweeping_status_offset: 0,
            scoring_scroll_offset: 0,
            max_scoring_scroll: 0,

            focus: PaneFocus::default(),
            previous_focus: PaneFocus::default(),
            should_quit: false,
        }
    }
}

impl AppState {
    // Handle an incoming event and update state accordingly
    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::StandingsUpdate(data) => {
                log::info!("Updating standings data");
                match StandingsResponse::from_json(&data) {
                    Ok(parsed_standings) => self.league_data = Some(parsed_standings),
                    Err(e) => log::error!("Failed to parse standings: {}", e),
                }
            }
            AppEvent::GamesUpdate(data) => {
                log::info!("Updating games data");
                match GamesResponse::from_json(&data) {
                    Ok(parsed_games) => {
                        self.games_data = Some(parsed_games);
                    }
                    Err(e) => log::error!("Failed to parse games: {}", e),
                }
            }
            AppEvent::Input(key_event) => {
                log::info!("Key event detected: {:?}", key_event);
                self.handle_key_event(key_event);
            }
            AppEvent::Tick => {
                self.sweeping_status_offset = self.sweeping_status_offset.wrapping_add(1);
            }
        }
    }

    /// Handle key event in normal mode
    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match (
            self.focus,
            self.selected_menu,
            key_event.code,
            key_event.modifiers,
        ) {
            // Ctrl + c quits no matter what
            (_, _, Char('c'), KeyModifiers::CONTROL) => self.should_quit = true,
            // q also quits no matter what
            (_, _, Char('q'), _) => self.should_quit = true,
            // In DatePicker mode, capture all input
            (PaneFocus::DatePicker, _, Char(c), _) => {
                self.date_input.is_valid = true; // reset status
                self.date_input.text.push(c);
            }
            // Key events for normal mode
            (PaneFocus::Content | PaneFocus::Menu, _, KeyCode::Tab, _) => {
                self.previous_focus = self.focus;
                self.focus = self.focus.switch();
            }
            (PaneFocus::Content | PaneFocus::Menu, _, KeyCode::Char(':'), _) => {
                self.focus = PaneFocus::DatePicker;
                self.previous_focus = self.focus;
                self.date_input.text.clear();
            }
            // (_, _, KeyCode::Char('r), _) => self.refresh(),
            // In menu pane
            (PaneFocus::Menu, _, KeyCode::Up | KeyCode::Char('k'), _) => {
                self.selected_menu = self.selected_menu.prev();
            }
            (PaneFocus::Menu, _, KeyCode::Down | KeyCode::Char('j'), _) => {
                self.selected_menu = self.selected_menu.next();
            }
            // In Games content pane
            (PaneFocus::Content, MenuFocus::Games, KeyCode::Up | KeyCode::Char('k'), _) => {
                self.scoring_scroll_offset = self.scoring_scroll_offset.saturating_sub(1);
            }
            (PaneFocus::Content, MenuFocus::Games, KeyCode::Down | KeyCode::Char('j'), _) => {
                self.scoring_scroll_offset = self
                    .scoring_scroll_offset
                    .saturating_add(1)
                    .min(self.max_scoring_scroll);
            }
            (PaneFocus::Content, MenuFocus::Games, KeyCode::Left | KeyCode::Char('h'), _) => {
                self.select_prev_game_index();
            }
            (PaneFocus::Content, MenuFocus::Games, KeyCode::Right | KeyCode::Char('l'), _) => {
                self.select_next_game_index();
            }
            // Toggle box score and overview
            (PaneFocus::Content, MenuFocus::Games, KeyCode::Char(','), _) => {
                todo!()
            }
            (PaneFocus::Content, MenuFocus::Games, KeyCode::Char('.'), _) => {
                todo!()
            }
            // Change dates quickly
            (PaneFocus::Content, MenuFocus::Games, KeyCode::Char('t'), _) => {
                todo!()
            }
            (PaneFocus::Content, MenuFocus::Games, KeyCode::Char('y'), _) => {
                todo!()
            }
            // In standings content pane
            (PaneFocus::Content, MenuFocus::Standings, KeyCode::Up | KeyCode::Char('k'), _) => {
                self.standings.move_selection(-1);
            }
            (PaneFocus::Content, MenuFocus::Standings, KeyCode::Down | KeyCode::Char('j'), _) => {
                self.standings.move_selection(1);
            }
            (PaneFocus::Content, MenuFocus::Standings, KeyCode::Left | KeyCode::Char('h'), _) => {
                self.standings.shift_standings_type(false);
            }
            (PaneFocus::Content, MenuFocus::Standings, KeyCode::Right | KeyCode::Char('l'), _) => {
                self.standings.shift_standings_type(true);
            }
            (PaneFocus::Content, MenuFocus::Standings, KeyCode::Char(',') | KeyCode::Char('.'), _) => {
                self.standings
                    .cycle_focus(matches!(key_event.code, KeyCode::Char('.')));
            }
            // In teams content pane
            (PaneFocus::Content, MenuFocus::Teams, _, _) => {
                todo!()
            }
            // Specific DatePicker commands
            // (PaneFocus::DatePicker, _, KeyCode::Enter, _) => {
            // if self.try_update_date_from_input().is_ok() {
            //     let previous_tab = self.previous_focus;
            //     guard.update_tab(previous_tab);
            //     handle_date_change(guard, network_requests).await;
            // }
            // }
            (PaneFocus::DatePicker, _, KeyCode::Right, _) => {
                self.move_date_selector_by_arrow(true);
            }
            (PaneFocus::DatePicker, _, KeyCode::Left, _) => {
                self.move_date_selector_by_arrow(false);
            }
            (PaneFocus::DatePicker, _, KeyCode::Esc, _) => {
                self.date_input.text.clear();
                self.focus = self.previous_focus;
            }
            (PaneFocus::DatePicker, _, KeyCode::Backspace, _) => {
                self.date_input.text.pop();
            }
            _ => {}
        }
    }
    pub fn move_date_selector_by_arrow(&mut self, right_arrow: bool) {
        let date = self.date_selector.set_date_with_arrows(right_arrow);
        self.date_input.text.clear();
        self.date_input.text.push_str(&date.to_string());
    }

    pub fn select_prev_game_index(&mut self) {
        let prev = self.selected_game_index;
        self.selected_game_index = self.prev_index(self.selected_game_index);
        if self.selected_game_index != prev {
            self.reset_scoring_scroll();
        }
    }
    pub fn select_next_game_index(&mut self) {
        let max_index = self.games_data.as_ref().map_or(0, |d| d.games.len());
        let prev = self.selected_game_index;
        self.selected_game_index = self.next_index(self.selected_game_index, max_index);
        if self.selected_game_index != prev {
            self.reset_scoring_scroll();
        }
    }
    pub fn prev_index(&self, index: usize) -> usize {
        index.saturating_sub(1)
    }
    pub fn next_index(&self, index: usize, max_index: usize) -> usize {
        (index + 1).min(max_index.saturating_sub(1))
    }

    /// Handle a mapped action.
    // pub fn handle_action(&mut self, action: Action) {
    //     if matches!(self.focus, PaneFocus::DatePicker) {
    //         handle_date_picker_action(action);
    //     }
    //     // Handle global actions
    //     match action {
    //         Action::CycleFocus => self.focus = self.focus.switch(),
    //         // Enter date selector
    //         Action::DateSelector => {
    //             self.focus = PaneFocus::DatePicker;
    //             self.input_mode = InputMode::EditingDate;
    //             self.date_input.text.clear();
    //         }
    //         Action::Quit => self.should_quit = true,
    //         Action::Refresh => {}
    //     }
    //     // Then handle pane specific actions
    //     match self.focus {
    //         PaneFocus::Content => self.handle_content_action(action),
    //         PaneFocus::DatePicker => self.handle_datepicker_action(action),
    //     }
    //     ////// Old
    //     match action {
    //         Action::MoveUp => self.move_selection(-1),
    //         Action::MoveDown => self.move_selection(1),
    //         Action::MoveLeft | Action::MoveRight => {
    //             let delta = if matches!(action, Action::MoveRight) {
    //                 1
    //             } else {
    //                 -1
    //             };
    //             match self.focus {
    //                 PaneFocus::Content => match self.selected_menu {
    //                     MenuFocus::Games => {
    //                         let len = self.games_data.as_ref().map_or(0, |d| d.games.len());
    //                         let prev = self.selected_game_index;
    //                         self.selected_game_index =
    //                             change_index(self.selected_game_index, delta, len);
    //                         if self.selected_game_index != prev {
    //                             self.reset_scoring_scroll();
    //                         }
    //                     }
    //                     MenuFocus::Standings => self.standings.shift_standings_type(delta == 1),
    //                     MenuFocus::Teams => {}
    //                 },
    //                 PaneFocus::DatePicker => {
    //                     let forward = if delta == 1 { true } else { false };
    //                     self.date_selector.set_date_with_arrows(forward);
    //                 }
    //                 _ => {}
    //             }
    //         }
    //         Action::NextContent | Action::PrevContent if self.focus == PaneFocus::Content => {
    //             self.standings
    //                 .cycle_focus(matches!(action, Action::NextContent));
    //         }
    //         _ => {}
    //     }
    // }

    // // When we are on a content pane
    // fn handle_content_action(&mut self, action: Action) {
    //     let delta = match action {
    //         Action::MoveRight => 1,
    //         Action::MoveLeft => -1,
    //         _ => 0,
    //     };
    //     match self.selected_menu {
    //         MenuFocus::Games => match action {
    //             Action::MoveUp => {
    //                 self.scoring_scroll_offset = self.scoring_scroll_offset.saturating_sub(1)
    //             }
    //             Action::MoveDown => {
    //                 self.scoring_scroll_offset = self
    //                     .scoring_scroll_offset
    //                     .saturating_add(1)
    //                     .min(self.max_scoring_scroll)
    //             }
    //             Action::MoveLeft | Action::MoveRight => {
    //                 let len = self.games_data.as_ref().map_or(0, |d| d.games.len());
    //                 let prev = self.selected_game_index;
    //                 self.selected_game_index = change_index(self.selected_game_index, delta, len);
    //                 if self.selected_game_index != prev {
    //                     self.reset_scoring_scroll();
    //                 }
    //             }
    //             _ => {}
    //         },
    //         MenuFocus::Standings => match action {
    //             Action::MoveUp => {
    //                 self.scoring_scroll_offset = self.scoring_scroll_offset.saturating_sub(1)
    //             }
    //             Action::MoveDown => {
    //                 self.scoring_scroll_offset = self
    //                     .scoring_scroll_offset
    //                     .saturating_add(1)
    //                     .min(self.max_scoring_scroll)
    //             }
    //             Action::MoveLeft | Action::MoveRight => {
    //                 let len = self.games_data.as_ref().map_or(0, |d| d.games.len());
    //                 let prev = self.selected_game_index;
    //                 self.selected_game_index = change_index(self.selected_game_index, delta, len);
    //                 if self.selected_game_index != prev {
    //                     self.reset_scoring_scroll();
    //                 }
    //             }
    //             _ => {}
    //         },
    //         MenuFocus::Teams => {}
    //     }
    // }

    /// Move the selection in the focused pane by `delta` (+1 = down, -1 = up).
    fn move_selection(&mut self, delta: i32) {
        match self.focus {
            PaneFocus::Menu => {
                self.selected_menu = if delta == 1 {
                    self.selected_menu.next()
                } else {
                    self.selected_menu.prev()
                };
            }
            PaneFocus::Content => match self.selected_menu {
                MenuFocus::Standings => self.standings.move_selection(delta),
                MenuFocus::Games => {
                    self.scoring_scroll_offset = if delta == 1 {
                        self.scoring_scroll_offset
                            .saturating_add(1)
                            .min(self.max_scoring_scroll)
                    } else {
                        self.scoring_scroll_offset.saturating_sub(1)
                    };
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn reset_scoring_scroll(&mut self) {
        self.scoring_scroll_offset = 0;
        self.max_scoring_scroll = 0;
    }
}

/// Change an index by delta within [0, len), capping at boundaries.
fn change_index(current: usize, delta: i32, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let new = current as i32 + delta;
    if new < 0 {
        current
    } else if new >= len as i32 {
        current
    } else {
        new as usize
    }
}
