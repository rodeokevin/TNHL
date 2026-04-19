use ratatui::widgets::TableState;

pub struct HelpState {
    pub table_state: TableState,
}

impl Default for HelpState {
    fn default() -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        Self { table_state }
    }
}

impl HelpState {
    pub fn row_down(&mut self) {
        self.table_state.scroll_down_by(1);
    }
    pub fn row_up(&mut self) {
        self.table_state.scroll_up_by(1);
    }
    pub fn reset(&mut self) {
        self.table_state.select(Some(0));
    }
    pub fn page_up(&mut self) {
        self.table_state.scroll_up_by(10);
    }
    pub fn page_down(&mut self) {
        self.table_state.scroll_down_by(10);
    }
}
