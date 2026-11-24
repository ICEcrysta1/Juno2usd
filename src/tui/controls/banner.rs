//! Banner：以 banner.txt 内容作为文字渲染（编译期内嵌，不分配）。

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget},
};

const BANNER: &str = include_str!("banner.txt");

pub struct Banner;

impl Widget for Banner {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(BANNER).render(area, buf);
    }
}
