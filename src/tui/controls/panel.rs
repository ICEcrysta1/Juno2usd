//! 圆角外盒 ╭─╮││╰─╯：只负责边框与标题，内部组件由调用方组合。

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, BorderType, Borders, Widget},
};

pub struct Panel<W: Widget> {
    title: String,
    inner: W,
}

impl<W: Widget> Panel<W> {
    pub fn new(title: impl Into<String>, inner: W) -> Self {
        Self {
            title: title.into(),
            inner,
        }
    }
}

impl<W: Widget> Widget for Panel<W> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(self.title);
        let inner = block.inner(area);
        block.render(area, buf);
        self.inner.render(inner, buf);
    }
}
