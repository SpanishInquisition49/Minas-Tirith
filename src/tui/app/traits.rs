use ratatui::widgets::ListState;

pub trait ListWidget<T> {
    fn items(&self) -> &[T];
    fn list_state(&self) -> &ListState;
    fn list_state_mut(&mut self) -> &mut ListState;

    fn selected(&self) -> Option<&T> {
        if let Some(index) = self.list_state().selected() {
            self.items().get(index)
        } else {
            None
        }
    }

    fn select_prev(&mut self) {
        let len = self.items().len();
        let i = match self.list_state().selected() {
            Some(i) if i > 0 => i - 1,
            Some(_) => len - 1,
            None => 0,
        };
        self.list_state_mut().select(Some(i));
    }

    fn select_next(&mut self) {
        let len = self.items().len();
        let i = match self.list_state().selected() {
            Some(i) if i + 1 < len => i + 1,
            Some(_) => 0,
            None => 0,
        };
        self.list_state_mut().select(Some(i));
    }
}
