use chrono::{NaiveDate, ParseError, Utc};
use chrono_tz::Tz;

/// Get user input for the date and store whether it's valid.
pub struct DateState {
    pub is_valid: bool,
    /// Current user input in the date picker
    pub text: String,
    /// Current date for the app
    pub date: NaiveDate,
    /// Used for selecting the date with arrow keys.
    pub selection_offset: i64,
}

impl DateState {
    /// Validate the input date
    pub fn validate_input(&mut self, tz: Tz) -> Result<NaiveDate, ParseError> {
        let input: String = self.text.drain(..).collect();
        let date = match input.as_str() {
            "t" | "today" => Ok(Utc::now().with_timezone(&tz).date_naive()),
            _ => NaiveDate::parse_from_str(input.as_str(), "%Y-%m-%d"),
        };
        self.is_valid = date.is_ok();
        date
    }
    /// Set the date from the validated input string from the date picker
    pub fn set_date_from_valid_input(&mut self, date: NaiveDate) {
        self.date = date;
        self.selection_offset = 0;
    }

    /// Set the date using Left/Right arrow keys to move a single day at a time
    pub fn set_date_with_arrows(&mut self, forward: bool) -> NaiveDate {
        match forward {
            true => self.selection_offset += 1,
            false => self.selection_offset -= 1,
        }
        self.date + chrono::Duration::days(self.selection_offset)
    }
    /// Format the data to be used in the title of a border;
    pub fn format_date_border_title(&self) -> String {
        self.date.format(" %B %d, %Y ").to_string()
    }
    pub fn move_date_selector_by_arrow(&mut self, right_arrow: bool) {
        let date = self.set_date_with_arrows(right_arrow);
        self.text.clear();
        self.text.push_str(&date.to_string());
    }
}

impl Default for DateState {
    fn default() -> Self {
        DateState {
            is_valid: true,
            text: String::new(),
            date: Utc::now().date_naive(),
            selection_offset: 0,
        }
    }
}
