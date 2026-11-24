//! 整屏布局：上盒(banner+工作目录) + 左盒(选择) + 右盒(信息) + 底部提示。
//! 只负责布局，框/列表/banner 均复用 controls。

use crate::tui::controls::{Banner, Button, Panel, SelectList};
use crate::tui::i18n::i18n;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Paragraph, Widget},
};

/// 页面所需数据（由 app 状态提供）
pub struct Page<'a> {
    pub version: &'a str,
    pub workspace: &'a str,
    pub files: &'a [String],
    pub selected: usize,
    pub input: &'a str,
    pub output: &'a str,
    pub parts: &'a str,
    pub connections: &'a str,
    pub materials: &'a str,
    pub size: &'a str,
    pub faces: Option<usize>,
    pub skel: Option<usize>,
    pub running: bool,
    pub progress: Option<(usize, usize)>,
}

impl Widget for Page<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(12),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(area);

        Panel::new(
            i18n!(ui_title, self.version),
            Top {
                workspace: self.workspace,
            },
        )
        .render(rows[0], buf);

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(30),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(rows[1]);

        let list = SelectList::new(self.files.to_vec(), self.selected);
        Panel::new(i18n!(ui_choose), list).render(cols[0], buf);

        Panel::new(
            i18n!(ui_info),
            Info {
                input: self.input,
                output: self.output,
                parts: self.parts,
                connections: self.connections,
                materials: self.materials,
                size: self.size,
                faces: self.faces,
                skel: self.skel,
            },
        )
        .render(cols[2], buf);

        let hints = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(34),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .split(rows[2]);
        Button::new(i18n!(ui_key_select))
            .alignment(Alignment::Left)
            .render(hints[0], buf);
        let mid = if self.running {
            match self.progress {
                Some((i, n)) => format!("{} {}/{}", i18n!(ui_running), i, n),
                None => i18n!(ui_running).to_string(),
            }
        } else {
            i18n!(ui_key_convert).to_string()
        };
        Button::new(mid)
            .focused(self.running)
            .alignment(Alignment::Center)
            .render(hints[1], buf);
        Button::new(i18n!(ui_key_quit))
            .alignment(Alignment::Right)
            .render(hints[2], buf);
    }
}

/// 上盒内容：banner + 工作目录
struct Top<'a> {
    workspace: &'a str,
}

impl Widget for Top<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let parts = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(area);
        Banner.render(parts[0], buf);
        Paragraph::new(format!("{}: {}", i18n!(ui_workspace), self.workspace))
            .alignment(Alignment::Center)
            .render(parts[1], buf);
    }
}

/// 右盒内容：转换前统计（零件/连接/材质/输入大小）+ 转换后面数/骨骼
struct Info<'a> {
    input: &'a str,
    output: &'a str,
    parts: &'a str,
    connections: &'a str,
    materials: &'a str,
    size: &'a str,
    faces: Option<usize>,
    skel: Option<usize>,
}

impl Widget for Info<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut text = format!(
            "{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}",
            i18n!(ui_input),
            self.input,
            i18n!(ui_output),
            self.output,
            i18n!(ui_parts),
            self.parts,
            i18n!(ui_connections),
            self.connections,
            i18n!(ui_materials),
            self.materials,
            i18n!(ui_size),
            self.size,
        );
        if let Some(f) = self.faces {
            text.push_str(&format!("\n{}: {} K", i18n!(ui_faces), f / 1000));
        }
        if let Some(s) = self.skel {
            text.push_str(&format!("\n{}: {}", i18n!(ui_skel), s));
        }
        // 与选择列表一致的 2 列左缩进，不贴边框
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(2), Constraint::Min(0)])
            .split(area);
        Paragraph::new(text).render(cols[1], buf);
    }
}

#[cfg(test)]
#[path = "../../../tests/src/tui/page/layout_test.rs"]
mod tests;
