mod core;
mod settings;
mod tui;

use std::path::Path;

/// 薄入口：有参数走 CLI 转换；无参数进入 TUI（不自动转换）。
fn main() {
    tui::i18n::init_from_env();
    let args: Vec<String> = std::env::args().collect();
    let Some(input) = args.get(1) else {
        println!("{}", tui::i18n::i18n!(ui_usage));
        return;
    };
    let dump = args.get(2).map(Path::new);
    let output = args.get(3).map(Path::new);

    match core::convert(Path::new(input), dump, output, |m| println!("{m}")) {
        Ok(r) => println!(
            "完成: 零件 {} 材质 {} 连接 {} | {} 顶点 {} 面 | {} 关节 {} 绑定",
            r.parts, r.materials, r.connections, r.vertices, r.faces, r.joints, r.bindings
        ),
        Err(e) => {
            eprintln!("转换失败: {e}");
            std::process::exit(1);
        }
    }
}
