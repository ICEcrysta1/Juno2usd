//! 转换编排入口：parse → shapes → normal → blend → skel → usda。
//!
//! 同步、无 UI 依赖；通过 progress 回调上报进度，供 CLI / TUI 复用。

use super::{normal, parse, shapes, skel, usda};
use crate::settings::Settings;
use shapes::RawMesh;
use shapes::algo::transform::Vec3;
use std::path::Path;

/// 转换结果统计
#[derive(Debug, Default)]
pub struct Report {
    pub materials: usize,
    pub parts: usize,
    pub connections: usize,
    pub joints: usize,
    pub bindings: usize,
    pub vertices: usize,
    pub faces: usize,
}

/// 执行完整转换。dump 为可选几何对照 JSON，output 为可选 USDA 产物。
pub fn convert(
    input: &Path,
    dump: Option<&Path>,
    output: Option<&Path>,
    mut progress: impl FnMut(&str),
) -> Result<Report, String> {
    let settings = Settings::load().map_err(|e| format!("加载 settings 失败: {e}"))?;
    let model = parse::parse_file(input, &settings)?;
    progress(&format!(
        "解析: 材质 {} | 零件 {} | 连接 {}",
        model.materials.len(),
        model.parts.len(),
        model.connections.connections.len()
    ));

    let mut meshes: Vec<RawMesh> = Vec::new();
    let mut part_ids: Vec<String> = Vec::new();
    let mut normals: Vec<Vec<Vec3>> = Vec::new();
    let mut report = Report {
        materials: model.materials.len(),
        parts: model.parts.len(),
        connections: model.connections.connections.len(),
        ..Default::default()
    };

    for (i, part) in model.parts.iter().enumerate() {
        if let Some(mesh) = shapes::generate(part, settings.mesh.segments) {
            let world = mesh.to_world(part);
            report.vertices += world.vertices.len();
            report.faces += world.faces.len();
            let ns = normal::compute(&world, settings.normals.sharp_edge_angle);
            part_ids.push(part.id.clone());
            meshes.push(world);
            normals.push(ns);
        }
        progress(&format!("几何+法线 {}/{}", i + 1, model.parts.len()));
    }

    normal::blend_joints(
        &meshes,
        &mut normals,
        &part_ids,
        &model.connections,
        settings.normals.sharp_edge_angle,
    );
    progress("接头法线混合完成");

    let skeleton = skel::build(&model.parts, &model.connections);
    report.joints = skeleton.joints.len();
    report.bindings = skeleton.part_joint.len();
    progress(&format!(
        "骨骼: {} 关节 {} 绑定",
        report.joints, report.bindings
    ));

    if let Some(dump) = dump {
        dump_json(dump, &meshes)?;
        progress(&format!("已导出对照: {}", dump.display()));
    }

    if let Some(output) = output {
        let prefix = input
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Model".to_string());
        usda::export(
            output,
            &prefix,
            &model.materials,
            &meshes,
            &normals,
            Some(&skeleton),
        )?;
        progress(&format!("USD 已导出: {}", output.display()));
    }

    Ok(report)
}

/// 手写 JSON 导出（几何对照用，避免引入序列化依赖）
fn dump_json(path: &Path, meshes: &[RawMesh]) -> Result<(), String> {
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
    std::fs::write(path, out).map_err(|e| format!("写入 JSON 失败: {e}"))
}
