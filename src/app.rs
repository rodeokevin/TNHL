use crate::state::app_state::AppState;

pub struct App {
    pub state: AppState,
}

impl App {
    pub fn new() -> Self {
        let app = Self {
            state: AppState::default(),
        };
        app
    }
}
