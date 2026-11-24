mod core;
mod settings;

use std::path::Path;

/// 薄入口：有参数走 CLI 转换；无参数不自动转换（TUI 落地后进入交互界面）。
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let Some(input) = args.get(1) else {
        println!("用法: juno2usd <input.xml> [dump.json] [output.usda]");
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
