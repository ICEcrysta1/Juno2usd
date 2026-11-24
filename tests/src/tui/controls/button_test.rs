use super::*;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

/// 左对齐默认 / 右对齐贴右 / 聚焦反色
#[test]
fn alignment_and_focus_reverse() {
    let area = Rect::new(0, 0, 20, 1);
    let mut buf = Buffer::empty(area);
    Button::new("ok").render(area, &mut buf);
    assert_eq!(buf[(1, 0)].symbol(), "o");

    let mut buf = Buffer::empty(area);
    Button::new("ok")
        .alignment(Alignment::Right)
        .render(area, &mut buf);
    assert_eq!(buf[(18, 0)].symbol(), "k");
    assert_eq!(buf[(19, 0)].symbol(), " ");

    let mut buf = Buffer::empty(area);
    Button::new("ok").focused(true).render(area, &mut buf);
    assert!(buf[(1, 0)].modifier.contains(Modifier::REVERSED));
}
