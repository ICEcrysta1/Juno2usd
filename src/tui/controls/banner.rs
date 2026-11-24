//! Banner：以 banner.txt 内容作为文字渲染（编译期内嵌，不分配）。

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    widgets::{Paragraph, Widget},
};

const BANNER: &str = include_str!("banner.txt");

pub struct Banner;

impl Widget for Banner {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let h = (BANNER.lines().count() as u16).min(area.height);
        let y = area.y + area.height.saturating_sub(h) / 2;
        let rect = Rect {
            x: area.x,
            y,
            width: area.width,
            height: h,
        };
        Paragraph::new(BANNER)
            .alignment(Alignment::Center)
            .render(rect, buf);
    }
}
