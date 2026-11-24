use super::*;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

/// Page 经重导出可渲染整屏
#[test]
fn reexport_page_renders() {
    let files = vec!["a.xml".to_string()];
    let p = Page {
        version: "0.2.0",
        workspace: "WS",
        files: &files,
        selected: 0,
        input: "i",
        output: "o",
        parts: "1",
        connections: "0",
        materials: "1",
        size: "1 KiB",
        faces: None,
        skel: None,
        running: false,
        progress: None,
    };
    let area = Rect::new(0, 0, 80, 24);
    let mut buf = Buffer::empty(area);
    p.render(area, &mut buf);
    assert_eq!(buf[(0, 0)].symbol(), "╭");
}
