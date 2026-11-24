use super::*;
use crate::tui::controls::Button;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

/// 圆角边框 + 居中标题 + 内部组件渲染在内区
#[test]
fn rounded_box_title_and_inner() {
    let area = Rect::new(0, 0, 20, 5);
    let mut buf = Buffer::empty(area);
    Panel::new("T", Button::new("ok")).render(area, &mut buf);
    assert_eq!(buf[(0, 0)].symbol(), "╭");
    assert_eq!(buf[(19, 0)].symbol(), "╮");
    assert_eq!(buf[(0, 4)].symbol(), "╰");
    assert_eq!(buf[(19, 4)].symbol(), "╯");
    let title: String = (0..20).map(|x| buf[(x, 0)].symbol()).collect();
    assert!(title.contains(" T "));
    // 内部按钮在内区左上 (1,1)
    assert_eq!(buf[(2, 1)].symbol(), "o");
}
