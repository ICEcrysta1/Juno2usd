//! TUI 应用：UI 优先。右栏 XML 解析后台执行，解析中显示旋转加载；
//! 快速切换时旧解析结果按 epoch 丢弃、立即解析新选中；解析结果按文件签名内存缓存，
//! 文件变更（len+mtime）即失效。不阻塞输入。

use crate::core::convert::{self, Report};
use crate::tui::page::Page;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
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
use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
    time::{Duration, SystemTime},
};

const SPINNER: [char; 10] = [
    '\u{280B}', '\u{2819}', '\u{2839}', '\u{2838}', '\u{283C}', '\u{2834}', '\u{2826}', '\u{2827}',
    '\u{2807}', '\u{280F}',
];

type Stats = (usize, usize, usize);
type Sig = (u64, SystemTime);

enum Msg {
    Progress(String),
    Done(Result<Report, String>),
    Parsed {
        index: usize,
        epoch: u64,
        stats: Option<Stats>,
        sig: Option<Sig>,
    },
    /// Input 目录变更（notify 事件推送）
    InputChanged,
}

fn fmt_size(len: u64) -> String {
    if len >= 1024 * 1024 {
        format!("{:.1} MiB", len as f64 / (1024.0 * 1024.0))
    } else {
        format!("{} KiB", len / 1024)
    }
}

fn sig_of(p: &PathBuf) -> Option<Sig> {
    std::fs::metadata(p)
        .ok()
        .map(|m| (m.len(), m.modified().unwrap_or(SystemTime::UNIX_EPOCH)))
}

struct App {
    paths: Vec<PathBuf>,
    names: Vec<String>,
    selected: usize,
    running: bool,
    progress: Option<(usize, usize)>,
    tx: mpsc::Sender<Msg>,
    rx: mpsc::Receiver<Msg>,
    parsed: Option<usize>,
    pending: Option<usize>,
    epoch: u64,
    frame: u64,
    cache: HashMap<PathBuf, (Sig, Stats)>,
    parts: Option<usize>,
    connections: Option<usize>,
    materials: Option<usize>,
    size_in: String,
    faces: Option<usize>,
    skel: Option<usize>,
    /// Input 目录监听器：存活期间事件经 tx 推送；创建失败为 None，退化为启动快照
    _watcher: Option<RecommendedWatcher>,
}

/// 扫描 Input 目录 XML 文件（排序）并生成显示名
fn scan_input() -> (Vec<PathBuf>, Vec<String>) {
    let mut paths: Vec<PathBuf> = std::fs::read_dir("Input")
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
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
    (paths, names)
}

impl App {
    fn new() -> Self {
        let (paths, names) = scan_input();
        let (tx, rx) = mpsc::channel();
        // Input 目录事件监听：notify 内部线程回调推送，非轮询
        let watcher = notify::recommended_watcher({
            let tx = tx.clone();
            move |res: notify::Result<notify::Event>| {
                if res.is_ok() {
                    let _ = tx.send(Msg::InputChanged);
                }
            }
        })
        .ok()
        .and_then(|mut w| {
            w.watch(Path::new("Input"), RecursiveMode::NonRecursive)
                .ok()
                .map(|()| w)
        });
        Self {
            paths,
            names,
            selected: 0,
            running: false,
            progress: None,
            tx,
            rx,
            parsed: None,
            pending: None,
            epoch: 0,
            frame: 0,
            cache: HashMap::new(),
            parts: None,
            connections: None,
            materials: None,
            size_in: "…".into(),
            faces: None,
            skel: None,
            _watcher: watcher,
        }
    }

    fn output_path(&self) -> Option<PathBuf> {
        let p = self.paths.get(self.selected)?;
        let stem = p.file_stem()?.to_string_lossy().into_owned();
        Some(PathBuf::from("Output").join(format!("{stem}.usda")))
    }

    fn apply(&mut self, s: Stats) {
        self.parts = Some(s.0);
        self.connections = Some(s.1);
        self.materials = Some(s.2);
    }

    /// Input 目录变更：重扫列表，按路径保留选中项，失效在途解析与已删文件缓存
    fn rescan(&mut self) {
        let old = self.paths.get(self.selected).cloned();
        (self.paths, self.names) = scan_input();
        self.selected = old
            .and_then(|p| self.paths.iter().position(|x| *x == p))
            .unwrap_or_else(|| self.selected.min(self.names.len().saturating_sub(1)));
        self.cache.retain(|p, _| self.paths.contains(p));
        self.epoch += 1;
        self.parsed = None;
        self.pending = None;
    }

    /// 每帧调用：缓存命中立即用；否则后台解析（旧 epoch 丢弃）
    fn refresh(&mut self) {
        let idx = self.selected;
        let Some(p) = self.paths.get(idx).cloned() else {
            self.size_in = "…".into();
            return;
        };
        if self.parsed != Some(idx) {
            self.parsed = Some(idx);
            self.faces = None;
            self.skel = None;
        }
        let sig = sig_of(&p);
        self.size_in = sig.map(|s| fmt_size(s.0)).unwrap_or_else(|| "…".into());

        // 缓存命中（签名一致）→ 立即显示
        if let Some((cs, stats)) = self.cache.get(&p)
            && sig == Some(*cs)
        {
            self.apply(*stats);
            self.pending = None;
            return;
        }
        // 已在解析当前选中 → 等待
        if self.pending == Some(idx) {
            return;
        }
        self.pending = Some(idx);
        self.epoch += 1;
        self.parts = None;
        self.connections = None;
        self.materials = None;
        let epoch = self.epoch;
        let tx = self.tx.clone();
        thread::spawn(move || {
            let stats = crate::settings::Settings::load()
                .ok()
                .and_then(|s| crate::core::parse::parse_file(&p, &s).ok())
                .map(|m| {
                    (
                        m.parts.len(),
                        m.connections.connections.len(),
                        m.materials.len(),
                    )
                });
            let _ = tx.send(Msg::Parsed {
                index: idx,
                epoch,
                stats,
                sig: sig_of(&p),
            });
        });
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
        self.running = true;
        self.progress = None;
        let tx = self.tx.clone();
        thread::spawn(move || {
            let r = convert::convert(&input, None, Some(&output), |m| {
                let _ = tx.send(Msg::Progress(m.to_string()));
            });
            let _ = tx.send(Msg::Done(r));
        });
    }

    fn drain(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
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
                    if let Ok(rep) = r {
                        self.faces = Some(rep.faces);
                        self.skel = Some(rep.joints);
                    }
                }
                Msg::Parsed {
                    index,
                    epoch,
                    stats,
                    sig,
                } => {
                    // 过期解析（用户已切换）直接丢弃
                    if epoch != self.epoch || index != self.selected {
                        continue;
                    }
                    self.pending = None;
                    if let Some(s) = stats {
                        if let Some(g) = sig {
                            self.cache.insert(self.paths[index].clone(), (g, s));
                        }
                        self.apply(s);
                    }
                }
                Msg::InputChanged => self.rescan(),
            }
        }
    }

    fn loading(&self) -> bool {
        self.pending == Some(self.selected) && self.parts.is_none()
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
        app.frame += 1;
        app.refresh();
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
            let spin = SPINNER[(app.frame % SPINNER.len() as u64) as usize].to_string();
            let num = |v: Option<usize>| {
                if app.loading() {
                    spin.clone()
                } else {
                    v.map(|n| n.to_string()).unwrap_or_else(|| "…".into())
                }
            };
            Page {
                version,
                workspace: &workspace,
                files: &app.names,
                selected: app.selected,
                input: &input,
                output: &output,
                parts: &num(app.parts),
                connections: &num(app.connections),
                materials: &num(app.materials),
                size: &app.size_in,
                faces: app.faces,
                skel: app.skel,
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
                KeyCode::Up => {
                    if !app.names.is_empty() {
                        // 顶部继续向上：循环回到列表底部
                        app.selected = app.selected.checked_sub(1).unwrap_or(app.names.len() - 1);
                    }
                }
                KeyCode::Down => {
                    if !app.names.is_empty() {
                        // 底部继续向下：循环回到列表顶部
                        app.selected = (app.selected + 1) % app.names.len();
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

#[cfg(test)]
#[path = "../../tests/src/tui/app_test.rs"]
mod tests;
