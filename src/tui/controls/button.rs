//! 按钮：内联标签，聚焦时反色，供底部操作条复用。

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

pub struct Button {
    label: String,
    focused: bool,
    align: Alignment,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            focused: false,
            align: Alignment::Left,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn alignment(mut self, align: Alignment) -> Self {
        self.align = align;
        self
    }
}

impl Widget for Button {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = if self.focused {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        let line = Line::from(Span::styled(format!(" {} ", self.label), style));
        Paragraph::new(line).alignment(self.align).render(area, buf);
    }
}

#[cfg(test)]
#[path = "../../../tests/src/tui/controls/button_test.rs"]
mod tests;
