use crate::{
    sources::{
        games::{boxscore::BoxscoreCommand, game_story::GameStoryCommand, games::GamesCommand},
        playoffs::{bracket::BracketCommand, series::SeriesCommand},
        standings::StandingsCommand,
        teams_stats::TeamStatsCommand,
    },
    state::{app_settings::AppSettings, app_state::AppState},
};
use chrono::{Datelike, Utc};
use tokio::sync::mpsc::Sender;

pub struct App {
    pub state: AppState,
    pub settings: AppSettings,
}

impl App {
    pub fn new(
        games_tx: Sender<GamesCommand>,
        standings_tx: Sender<StandingsCommand>,
        boxscore_tx: Sender<BoxscoreCommand>,
        game_story_tx: Sender<GameStoryCommand>,
        team_stats_tx: Sender<TeamStatsCommand>,
        bracket_tx: Sender<BracketCommand>,
        series_tx: Sender<SeriesCommand>,
    ) -> Self {
        let mut app = Self {
            state: AppState::new(
                games_tx,
                standings_tx,
                boxscore_tx,
                game_story_tx,
                team_stats_tx,
                bracket_tx,
                series_tx,
            ),
            settings: AppSettings::load_from_file(),
        };
        app.configure();
        app
    }

    /// Run any final configuration
    fn configure(&mut self) {
        self.set_all_datepickers_to_today();
        self.set_time_zone();

        // Apply the favorite team (if configured) as the default team shown on
        // the Team Stats page. The team stats source is initialized with the
        // same team in `main.rs`, so no command needs to be sent here.
        if let Some(favorite) = self.settings.favorite_team {
            log::info!("Favorite team configured: {favorite}");
            self.state.team_stats.team_picker.current_team = favorite;
        }

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
        self.state.date_state.date = today;
        self.state.date_state.year = today.year();
    }

    /// Set the timezone in the state to the settings' timezone
    fn set_time_zone(&mut self) {
        self.state.timezone = self.settings.timezone;
    }
}
