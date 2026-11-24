use super::*;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

/// 四个组件经重导出可组合渲染
#[test]
fn reexports_compose() {
    let area = Rect::new(0, 0, 30, 8);
    let mut buf = Buffer::empty(area);
    Panel::new("T", SelectList::new(vec!["x".into()], 0)).render(area, &mut buf);
    assert_eq!(buf[(0, 0)].symbol(), "╭");
    let mut buf2 = Buffer::empty(Rect::new(0, 0, 30, 8));
    Banner.render(Rect::new(0, 0, 30, 8), &mut buf2);
    Button::new("b").render(Rect::new(0, 0, 10, 1), &mut buf2);
    assert_eq!(buf2[(1, 0)].symbol(), "b");
}
