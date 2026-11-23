//! USD 导出编排：按材质分组合并网格（与旧导出器语义一致），
//! 退化面过滤 + 局部顶点紧凑化；不做几何生成与法线计算。

use super::mesh::CompactMesh;
use super::{material, mesh, stage};
use crate::core::parse::material::{Material, clean_name};
use crate::core::shapes::RawMesh;
use std::collections::HashMap;
use std::path::Path;

/// 导出 USDA：根 Xform + 每材质一个 Mesh + UsdPreviewSurface
pub fn export(
    output: &Path,
    prefix: &str,
    materials: &[Material],
    meshes: &[RawMesh],
) -> Result<(), String> {
    let (stage, root) = stage::create(prefix)?;

    // 按材质分组（保持首次出现顺序）
    let mut groups: Vec<(String, Vec<&RawMesh>)> = Vec::new();
    for mesh in meshes {
        match groups
            .iter_mut()
            .find(|(name, _)| *name == mesh.material_name)
        {
            Some((_, list)) => list.push(mesh),
            None => groups.push((mesh.material_name.clone(), vec![mesh])),
        }
    }

    for (i, (mat_name, group)) in groups.iter().enumerate() {
        let compact = compact(group);
        if compact.faces.is_empty() {
            continue;
        }
        // 空材质名兜底（与旧导出器一致），且不创建/绑定材质
        let mesh_name = if mat_name.is_empty() {
            format!("Mesh_{i}")
        } else {
            mat_name.clone()
        };
        let mesh_path = format!("/{root}/{mesh_name}");
        mesh::write(&stage, &mesh_path, &compact)?;
        if !mat_name.is_empty() {
            let data = materials.iter().find(|m| clean_name(&m.name) == *mat_name);
            material::define_and_bind(&stage, &root, mat_name, data, &mesh_path)?;
        }
    }

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    stage
        .root_layer()
        .borrow()
        .export(output.to_string_lossy())
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// 合并同材质网格：过滤退化面，紧凑化被引用的顶点
fn compact(group: &[&RawMesh]) -> CompactMesh {
    let mut points: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut faces: Vec<[i32; 3]> = Vec::new();

    for mesh in group {
        let mut remap: HashMap<usize, i32> = HashMap::new();
        for f in &mesh.faces {
            if f[0] == f[1] || f[1] == f[2] || f[0] == f[2] {
                continue;
            }
            let mut nf = [0i32; 3];
            for k in 0..3 {
                nf[k] = *remap.entry(f[k]).or_insert_with(|| {
                    points.push(mesh.vertices[f[k]].map(|c| c as f32));
                    uvs.push(mesh.uvs[f[k]].map(|c| c as f32));
                    (points.len() - 1) as i32
                });
            }
            faces.push(nf);
        }
    }

    CompactMesh { points, uvs, faces }
}
