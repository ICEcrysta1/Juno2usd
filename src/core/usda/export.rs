//! USD 导出编排：按材质分组、紧凑化写出；不做几何生成与法线计算。
//! 法线由调用方逐零件计算并经连接混合后传入。

use super::mesh::CompactMesh;
use super::{material, mesh, stage};
use crate::core::parse::material::{Material, clean_name};
use crate::core::shapes::RawMesh;
use crate::core::shapes::algo::transform::Vec3;
use std::collections::HashMap;
use std::path::Path;

/// 导出 USDA：根 Xform + 每材质一个 Mesh + UsdPreviewSurface + faceVarying 法线。
/// normals 与 meshes 一一对应（世界空间）。
pub fn export(
    output: &Path,
    prefix: &str,
    materials: &[Material],
    meshes: &[RawMesh],
    normals: &[Vec<Vec3>],
) -> Result<(), String> {
    let (stage, root) = stage::create(prefix)?;

    // 按材质分组（保持首次出现顺序），记录网格索引
    let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
    for (i, m) in meshes.iter().enumerate() {
        match groups.iter_mut().find(|(name, _)| *name == m.material_name) {
            Some((_, list)) => list.push(i),
            None => groups.push((m.material_name.clone(), vec![i])),
        }
    }

    for (i, (mat_name, idxs)) in groups.iter().enumerate() {
        let group_meshes: Vec<&RawMesh> = idxs.iter().map(|&i| &meshes[i]).collect();
        let group_normals: Vec<&[Vec3]> = idxs.iter().map(|&i| normals[i].as_slice()).collect();
        let compact = compact(&group_meshes, &group_normals);
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

/// 合并同材质网格：过滤退化面，紧凑化被引用的顶点（保留各自 UV），法线随面携带
fn compact(meshes: &[&RawMesh], normals: &[&[Vec3]]) -> CompactMesh {
    let mut points: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut faces: Vec<[i32; 3]> = Vec::new();
    let mut out_normals: Vec<[f32; 3]> = Vec::new();

    for (mi, m) in meshes.iter().enumerate() {
        let ns = normals[mi];
        let mut remap: HashMap<usize, i32> = HashMap::new();
        for (fi, f) in m.faces.iter().enumerate() {
            if f[0] == f[1] || f[1] == f[2] || f[0] == f[2] {
                continue;
            }
            let mut nf = [0i32; 3];
            for k in 0..3 {
                nf[k] = *remap.entry(f[k]).or_insert_with(|| {
                    points.push(m.vertices[f[k]].map(|c| c as f32));
                    uvs.push(m.uvs[f[k]].map(|c| c as f32));
                    (points.len() - 1) as i32
                });
            }
            faces.push(nf);
            for k in 0..3 {
                out_normals.push(ns[fi * 3 + k].map(|c| c as f32));
            }
        }
    }

    CompactMesh {
        points,
        uvs,
        faces,
        normals: out_normals,
    }
}
