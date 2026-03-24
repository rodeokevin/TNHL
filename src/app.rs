use crate::state::{app_settings::AppSettings, app_state::AppState};
use chrono::Utc;

pub struct App {
    pub state: AppState,
    pub settings: AppSettings,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            state: AppState::default(),
            settings: AppSettings::load_from_file(),
        };
        app.configure();
        app
    }

    /// Run any final configuration that might need to access multiple parts of state.
    fn configure(&mut self) {
        self.set_all_datepickers_to_today();
        // self.state.standings.favorite_team = self.settings.favorite_team;

        // override log level if set
        if let Some(level) = self.settings.log_level {
            log::set_max_level(level);
            tui_logger::set_default_level(level);
        }
    }

    /// Sync date pickers using the correct timezone.
    fn set_all_datepickers_to_today(&mut self) {
        let today = Utc::now()
            .with_timezone(&self.settings.timezone)
            .date_naive();
        self.state.date_selector.date = today;
    }
}
