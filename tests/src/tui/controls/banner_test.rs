use super::*;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

/// banner 垂直居中渲染内嵌文字
#[test]
fn banner_centered_vertically() {
    let area = Rect::new(0, 0, 80, 12);
    let mut buf = Buffer::empty(area);
    Banner.render(area, &mut buf);
    let lines = BANNER.lines().count() as u16;
    let y0 = (12 - lines) / 2;
    let row: String = (0..80).map(|x| buf[(x, y0)].symbol()).collect();
    assert!(row.contains("██"), "banner 首行应居中渲染");
    let top: String = (0..80).map(|x| buf[(x, 0)].symbol()).collect();
    assert!(top.trim().is_empty(), "顶部留白");
}
