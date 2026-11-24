//! 按钮：内联标签，聚焦时反色，供底部操作条复用。

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

pub struct Button {
    label: String,
    focused: bool,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            focused: false,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
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
        Line::from(Span::styled(format!(" {} ", self.label), style)).render(area, buf);
    }
}
