//! TUI 应用：状态 + 按键 + 后台转换，驱动 page 渲染。

use crate::core::convert::{self, Report};
use crate::tui::page::Page;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    widgets::Widget,
};
use std::{io, path::PathBuf, sync::mpsc, thread, time::Duration};

enum Msg {
    Progress(String),
    Done(Result<Report, String>),
}

struct App {
    paths: Vec<PathBuf>,
    names: Vec<String>,
    selected: usize,
    running: bool,
    progress: Option<(usize, usize)>,
    rx: Option<mpsc::Receiver<Msg>>,
    components: String,
    skel: String,
    size: String,
}

impl App {
    fn new() -> Self {
        let mut paths: Vec<PathBuf> = std::fs::read_dir("Input")
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("xml")))
            .collect();
        paths.sort();
        let names = paths
            .iter()
            .map(|p| {
                p.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        Self {
            paths,
            names,
            selected: 0,
            running: false,
            progress: None,
            rx: None,
            components: "…".into(),
            skel: "…".into(),
            size: "…".into(),
        }
    }

    fn output_path(&self) -> Option<PathBuf> {
        let p = self.paths.get(self.selected)?;
        let stem = p.file_stem()?.to_string_lossy().into_owned();
        Some(PathBuf::from("Output").join(format!("{stem}_rust.usda")))
    }

    fn start(&mut self) {
        if self.running || self.names.is_empty() {
            return;
        }
        let Some(input) = self.paths.get(self.selected).cloned() else {
            return;
        };
        let Some(output) = self.output_path() else {
            return;
        };
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        self.running = true;
        self.progress = None;
        thread::spawn(move || {
            let r = convert::convert(&input, None, Some(&output), |m| {
                let _ = tx.send(Msg::Progress(m.to_string()));
            });
            let _ = tx.send(Msg::Done(r));
        });
    }

    fn drain(&mut self) {
        let Some(rx) = &self.rx else { return };
        while let Ok(msg) = rx.try_recv() {
            match msg {
                Msg::Progress(m) => {
                    if let Some(rest) = m.strip_prefix("几何+法线 ")
                        && let Some((a, b)) = rest.split_once('/')
                        && let (Ok(i), Ok(n)) = (a.parse(), b.parse())
                    {
                        self.progress = Some((i, n));
                    }
                }
                Msg::Done(r) => {
                    self.running = false;
                    self.progress = None;
                    match r {
                        Ok(rep) => {
                            self.components = format!("{} K", rep.faces / 1000);
                            self.skel = rep.joints.to_string();
                            self.size = self
                                .output_path()
                                .and_then(|p| std::fs::metadata(p).ok())
                                .map(|m| format!("{} KiB", m.len() / 1024))
                                .unwrap_or_else(|| "…".into());
                        }
                        Err(_) => {
                            self.components = "…".into();
                            self.skel = "…".into();
                            self.size = "ERR".into();
                        }
                    }
                }
            }
        }
    }
}

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();
    let workspace = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let version = env!("CARGO_PKG_VERSION");

    let result = loop {
        app.drain();
        terminal.draw(|f| {
            let input = app
                .paths
                .get(app.selected)
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            let output = app
                .output_path()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            Page {
                version,
                workspace: &workspace,
                files: &app.names,
                selected: app.selected,
                input: &input,
                output: &output,
                components: &app.components,
                skel: &app.skel,
                size: &app.size,
                running: app.running,
                progress: app.progress,
            }
            .render(f.area(), f.buffer_mut());
        })?;

        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key) = event::read()?
        {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
                KeyCode::Up => app.selected = app.selected.saturating_sub(1),
                KeyCode::Down => {
                    if !app.names.is_empty() {
                        app.selected = (app.selected + 1).min(app.names.len() - 1);
                    }
                }
                KeyCode::Enter => app.start(),
                _ => {}
            }
        }
    };

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}
