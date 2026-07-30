pub trait Cyclable: Sized {
    fn next(&self) -> Self;
    fn prev(&self) -> Self;
}

pub trait Focusable {
    type Focus: Cyclable;
    fn current_focus(&self) -> Self::Focus;
    fn current_focus_mut(&mut self) -> &mut Self::Focus;

    fn focus_next(&mut self) {
        let next = self.current_focus().next();
        *self.current_focus_mut() = next;
    }

    fn focus_prev(&mut self) {
        let prev = self.current_focus().prev();
        *self.current_focus_mut() = prev;
    }
}

pub trait Selectable {
    type Item;
    fn items(&self) -> &[Self::Item];
    fn selected_index(&self) -> Option<usize>;
    fn selected_index_mut(&mut self) -> &mut Option<usize>;

    fn select_next(&mut self) {
        let len = self.items().len();
        let next = match self.selected_index() {
            Some(i) if i + 1 < len => Some(i + 1),
            Some(_) if len != 0 => Some(0),
            None if len != 0 => Some(0),
            _ => None,
        };
        *self.selected_index_mut() = next;
    }

    fn select_prev(&mut self) {
        let len = self.items().len();
        let prev = match self.selected_index() {
            Some(i) if i > 0 => Some(i - 1),
            Some(_) if len != 0 => Some(len - 1),
            None if len != 0 => Some(0),
            _ => None,
        };
        *self.selected_index_mut() = prev;
    }
}
