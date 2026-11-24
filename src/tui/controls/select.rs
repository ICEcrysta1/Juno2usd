//! 选择列表：仅负责选项与 ❱ 高亮渲染，不含外盒（外盒由 Panel 组合）。
//! 选中索引由调用方（app）管理。

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
    pub fn new(items: Vec<String>, selected: usize) -> Self {
        Self { items, selected }
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

#[cfg(test)]
#[path = "../../../tests/src/tui/controls/select_test.rs"]
mod tests;
