//! 选择列表：仅负责选项与 ❱ 高亮，不含外盒（外盒由 Panel 组合）。

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{List, ListItem, ListState, StatefulWidget, Widget},
};

pub struct SelectList {
    items: Vec<String>,
    selected: usize,
}

impl SelectList {
    pub fn new(items: Vec<String>) -> Self {
        Self { items, selected: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn select(&mut self, i: usize) {
        if i < self.items.len() {
            self.selected = i;
        }
    }

    pub fn next(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + 1).min(self.items.len() - 1);
        }
    }

    pub fn prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }
}

impl Widget for SelectList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self.items.into_iter().map(ListItem::new).collect();
        let mut state = ListState::default().with_selected(Some(self.selected));
        let list = List::new(items)
            .highlight_symbol("❱ ")
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        StatefulWidget::render(list, area, buf, &mut state);
    }
}
