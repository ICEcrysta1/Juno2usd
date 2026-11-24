use super::*;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

fn page(faces: Option<usize>, running: bool) -> Page<'static> {
    Page {
        version: "0.2.0",
        workspace: "WS",
        files: &[],
        selected: 0,
        input: "Input/a.xml",
        output: "Output/a.usda",
        parts: "3",
        connections: "2",
        materials: "1",
        size: "1.0 MiB",
        faces,
        skel: Some(7),
        running,
        progress: running.then_some((1, 2)),
    }
}

fn row(buf: &Buffer, y: u16, x0: u16, x1: u16) -> String {
    (x0..x1).map(|x| buf[(x, y)].symbol()).collect()
}

/// 三盒布局：上盒12行、左盒30列、1列间隔、右盒余宽；圆角边框
#[test]
fn three_box_layout_geometry() {
    let area = Rect::new(0, 0, 100, 30);
    let mut buf = Buffer::empty(area);
    page(None, false).render(area, &mut buf);
    assert_eq!(buf[(0, 0)].symbol(), "╭");
    assert_eq!(buf[(99, 0)].symbol(), "╮");
    assert_eq!(buf[(0, 11)].symbol(), "╰");
    assert_eq!(buf[(0, 12)].symbol(), "╭");
    assert_eq!(buf[(29, 12)].symbol(), "╮");
    assert_eq!(buf[(0, 28)].symbol(), "╰");
    assert_eq!(buf[(29, 28)].symbol(), "╯");
    assert_eq!(buf[(31, 12)].symbol(), "╭");
    assert_eq!(buf[(99, 12)].symbol(), "╮");
    assert_eq!(buf[(29, 15)].symbol(), "│");
    assert_eq!(buf[(30, 15)].symbol(), " ");
    assert_eq!(buf[(31, 15)].symbol(), "│");
}

/// 右盒转换后追加面数(K)/骨骼；底部左/右提示铺满整行、运行中显示进度
#[test]
fn info_extra_lines_and_bottom_hints() {
    let area = Rect::new(0, 0, 100, 30);
    let mut buf = Buffer::empty(area);
    page(Some(12345), true).render(area, &mut buf);
    let info: String = (13..29)
        .map(|y| row(&buf, y, 32, 100))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(info.contains("12 K"), "面数应以 K 显示");
    assert!(info.contains(": 7"), "骨骼数应显示");
    assert!(info.contains("1.0 MiB"));
    // 底部：左提示贴左、右提示贴右、中间进度
    let bottom: Vec<&str> = (0..100).map(|x| buf[(x, 29)].symbol()).collect();
    let first = bottom.iter().position(|s| *s != " ").unwrap();
    let last = bottom.iter().rposition(|s| *s != " ").unwrap();
    assert!(first <= 2, "左提示应贴左");
    assert!(last >= 97, "右提示应贴右");
    assert!(row(&buf, 29, 34, 67).contains("1/2"));
}
