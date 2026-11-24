use super::*;
use ratatui::{buffer::Buffer, layout::Rect, style::Modifier, widgets::Widget};

/// 选中行 ❱ 高亮反色，未选中行无高亮
#[test]
fn selected_row_highlight() {
    let area = Rect::new(0, 0, 10, 3);
    let mut buf = Buffer::empty(area);
    SelectList::new(vec!["a".into(), "b".into(), "c".into()], 1).render(area, &mut buf);
    assert!(buf[(0, 1)].modifier.contains(Modifier::REVERSED));
    assert_ne!(buf[(0, 1)].symbol(), " ");
    assert_eq!(buf[(2, 1)].symbol(), "b");
    // List 为高亮符号在所有行预留宽度
    assert_eq!(buf[(2, 0)].symbol(), "a");
    assert!(!buf[(0, 0)].modifier.contains(Modifier::REVERSED));
}
