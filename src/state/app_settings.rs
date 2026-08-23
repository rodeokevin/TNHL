use crate::config::ConfigFile;
use crate::models::TeamAbbrev;
use chrono_tz::Tz;

#[derive(Debug, Default, Clone)]
pub struct AppSettings {
    pub favorite_team: Option<TeamAbbrev>,
    pub timezone: Tz,
    pub timezone_abbreviation: String,
    pub log_level: Option<log::LevelFilter>,
}

impl AppSettings {
    /// If config file can't be loaded just print an error message but don't block starting app
    pub fn load_from_file() -> Self {
        ConfigFile::load_from_file()
            .unwrap_or_else(|err| {
                log::error!("Could not load config file: {err}");
                ConfigFile::default()
            })
            .into()
    }
}
