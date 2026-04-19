use crate::banner::BANNER;
use crate::config::ConfigFile;
use ratatui::layout::{Constraint, Flex, Layout};
use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, Row, Table, TableState};

const HEADER: &[&str; 2] = &["Command", "Key"];
const GENERAL_DOCS: &[&[&str; 2]; 8] = &[
    &["Exit help", "esc"],
    &["Move down", "j/↓"],
    &["Move up", "k/↑"],
    &["Page down", "shift + j/↓"],
    &["Page up", "shift + k/↑"],
    &["Select menu", "1/2/3/4"],
    &["Show/hide menu", "m"],
    &["Quit TNHL", "q/ctrl + c"],
];
const GAMES_DOCS: &[&[&str; 2]; 10] = &[
    &["Games", " "],
    &["Next game", "l/→"],
    &["Previous game", "h/←"],
    &["Scroll down", "j/↓"],
    &["Scroll up", "k/↑"],
    &["Page down", "shift + j/↓"],
    &["Page up", "shift + k/↑"],
    &["Next view", "<"],
    &["Previous view", ">"],
    &["Select date", ":"],
];
const STANDINGS_DOCS: &[&[&str; 2]; 10] = &[
    &["Standings", " "],
    &["Next standings", "l/→"],
    &["Previous standings", "h/←"],
    &["Move down", "j/↓"],
    &["Move up", "k/↑"],
    &["Page down", "shift + j/↓"],
    &["Page up", "shift + k/↑"],
    &["Next division/conference", ">"],
    &["Previous division/conference", "<"],
    &["Select date", ":"],
];
const TEAM_STATS_DOCS: &[&[&str; 2]; 8] = &[
    &["Team Stats", " "],
    &["Move down", "j/↓"],
    &["Move up", "k/↑"],
    &["Page down", "shift + j/↓"],
    &["Page up", "shift + k/↑"],
    &["Toggle skaters/goalies", "</>"],
    &["Select year", ":"],
    &["Select team", "t"],
];
const PLAYOFFS_DOCS: &[&[&str; 2]; 11] = &[
    &["Playoffs", " "],
    &["Move down", "j/↓"],
    &["Move up", "k/↑"],
    &["Move left", "h/←"],
    &["Move right", "l/→"],
    &["Page down", "shift + j/↓"],
    &["Page up", "shift + k/↑"],
    &["Page left", "shift + h/←"],
    &["Page right", "shift + l/→"],
    &["Select series", "enter series letter"],
    &["Select year", ":"],
];
const PLAYERS_DOCS: &[&[&str; 2]; 2] = &[&["Players", " "], &["To be implemented", ""]];
const STATSTICS_DOCS: &[&[&str; 2]; 2] = &[&["Statistics", " "], &["To be implemented", ""]];

#[derive(Clone, Copy, Eq, PartialEq)]
enum RowType {
    Header,
    SubHeader,
    Row,
}

/// Used to keep track of row type for styling.
struct HelpRow {
    row_type: RowType,
    text: Vec<String>,
}

pub struct HelpWidget {}

impl StatefulWidget for HelpWidget {
    type State = TableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Create a one-column table to avoid flickering due to non-determinism when
        // resolving constraints on widths of table columns.
        let format_row = |r: &[&str; 2]| -> HelpRow {
            let row_type = if r[1].parse::<u8>().is_ok() {
                RowType::Header
            } else if r[1] == " " {
                RowType::SubHeader
            } else {
                RowType::Row
            };
            HelpRow {
                row_type,
                text: vec![format!("{:30}{:15}", r[0], r[1])],
            }
        };
        let header_style = Style::new().bold().underlined();
        let sub_header_style = Style::new().bold().underlined();
        let help_menu_style = Style::default();

        let header = Row::new(format_row(HEADER).text)
            .height(1)
            .bottom_margin(0)
            .style(header_style);

        let docs = build_docs();

        let rows = docs
            .iter()
            .map(|d| format_row(d))
            .map(|item| match item.row_type {
                RowType::Header => Row::new(item.text).style(header_style),
                RowType::SubHeader => Row::new(item.text).style(sub_header_style),
                RowType::Row => Row::new(item.text).style(help_menu_style),
            });

        let [table, banner] = Layout::horizontal([Constraint::Length(50), Constraint::Length(15)])
            .flex(Flex::Legacy)
            .margin(1)
            .horizontal_margin(2)
            .areas(area);

        let selected_style = Style::new().bg(Color::DarkGray).bold();
        StatefulWidget::render(
            Table::new(rows, [Constraint::Percentage(100)])
                .header(header)
                .style(help_menu_style)
                .row_highlight_style(selected_style),
            table,
            buf,
            state,
        );

        let config_file = if let Some(path) = ConfigFile::get_config_location() {
            path.to_string_lossy().to_string()
        } else {
            "not found".to_string()
        };
        Paragraph::new(format!(
            "{}\nv. {}\n\nconfig:\n{}",
            BANNER,
            env!("CARGO_PKG_VERSION"),
            config_file
        ))
        .centered()
        .render(banner, buf);
    }
}

fn build_docs() -> Vec<&'static [&'static str; 2]> {
    let mut docs = GENERAL_DOCS.to_vec();
    docs.extend_from_slice(GAMES_DOCS);
    docs.extend_from_slice(STANDINGS_DOCS);
    docs.extend_from_slice(TEAM_STATS_DOCS);
    docs.extend_from_slice(PLAYOFFS_DOCS);
    docs.extend_from_slice(PLAYERS_DOCS);
    docs.extend_from_slice(STATSTICS_DOCS);

    docs
}
