mod core;
mod settings;

use core::shapes::{self, RawMesh};
use std::path::Path;

/// 临时验证入口：解析 XML 并生成每个零件网格（世界空间），
/// 可导出 JSON 供与旧实现对照；TUI 落地后替换。
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("Input/Test-Juno2OBJ.xml");

    let settings = settings::Settings::load().expect("加载 settings 失败");
    let model = shapes::parse_file(Path::new(input), &settings).expect("解析 XML 失败");
    println!(
        "材质 {} | 零件 {} | 连接 {}",
        model.materials.len(),
        model.parts.len(),
        model.connections.connections.len()
    );

    let mut meshes: Vec<RawMesh> = Vec::new();
    let mut total_v = 0;
    let mut total_f = 0;
    for part in &model.parts {
        match shapes::generate(part, settings.mesh.segments) {
            Some(mesh) => {
                let world = mesh.to_world(part);
                println!(
                    "  {} ({:?}): {} 顶点, {} 面",
                    part.id,
                    part.part_type,
                    world.vertices.len(),
                    world.faces.len()
                );
                total_v += world.vertices.len();
                total_f += world.faces.len();
                meshes.push(world);
            }
            None => println!("  {} ({:?}): 无几何", part.id, part.part_type),
        }
    }
    println!("总计: {total_v} 顶点, {total_f} 面");

    if let Some(dump_path) = args.get(2) {
        dump_json(dump_path, &meshes);
        println!("已导出: {dump_path}");
    }
}

/// 手写 JSON 导出（临时对照用，避免引入序列化依赖）
fn dump_json(path: &str, meshes: &[RawMesh]) {
    let mut out = String::from("{\n");
    for (i, m) in meshes.iter().enumerate() {
        out.push_str(&format!("  \"{}\": {{\n    \"vertices\": [", m.part_id));
        for (j, v) in m.vertices.iter().enumerate() {
            if j > 0 {
                out.push(',');
            }
            out.push_str(&format!("[{:.10},{:.10},{:.10}]", v[0], v[1], v[2]));
        }
        out.push_str("],\n    \"faces\": [");
        for (j, f) in m.faces.iter().enumerate() {
            if j > 0 {
                out.push(',');
            }
            out.push_str(&format!("[{},{},{}]", f[0], f[1], f[2]));
        }
        out.push_str("]\n  }");
        if i + 1 < meshes.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("}\n");
    std::fs::write(path, out).expect("写入 JSON 失败");
}
