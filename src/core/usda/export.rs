//! USD 导出编排：按 (关节, 材质) 分组、紧凑化写出；不做几何生成与法线计算。
//! 法线由调用方逐零件计算并经连接混合后传入。有骨骼时写 SkelRoot/Skeleton 并绑定。

use super::mesh::CompactMesh;
use super::{material, mesh, skel as usd_skel, stage};
use crate::core::parse::material::{Material, clean_name};
use crate::core::shapes::RawMesh;
use crate::core::shapes::algo::transform::Vec3;
use crate::core::skel::Skeleton;
use std::collections::HashMap;
use std::path::Path;

/// 导出 USDA：根 Xform + Mesh + UsdPreviewSurface + faceVarying 法线。
/// skeleton 非空且有关节时启用 UsdSkel（SkelRoot 下按 (关节,材质) 分组并绑定）。
/// normals 与 meshes 一一对应（世界空间）。
pub fn export(
    output: &Path,
    prefix: &str,
    materials: &[Material],
    meshes: &[RawMesh],
    normals: &[Vec<Vec3>],
    skeleton: Option<&Skeleton>,
) -> Result<(), String> {
    let (stage, root) = stage::create(prefix)?;

    let active = skeleton.filter(|s| !s.joints.is_empty());
    let (parent, skel_path) = if let Some(sk) = active {
        let sp = usd_skel::define(&stage, &format!("/{root}"), sk)?;
        (format!("/{root}/SkelRoot"), Some(sp))
    } else {
        (format!("/{root}"), None)
    };

    // 按 (关节, 材质) 分组（保持首次出现顺序），记录网格索引
    let mut groups: Vec<(Option<usize>, String, Vec<usize>)> = Vec::new();
    for (i, m) in meshes.iter().enumerate() {
        let j = active.and_then(|sk| sk.part_joint.get(&m.part_id).copied());
        match groups
            .iter_mut()
            .find(|(gj, gn, _)| *gj == j && *gn == m.material_name)
        {
            Some((_, _, list)) => list.push(i),
            None => groups.push((j, m.material_name.clone(), vec![i])),
        }
    }

    for (i, (j, mat_name, idxs)) in groups.iter().enumerate() {
        let group_meshes: Vec<&RawMesh> = idxs.iter().map(|&i| &meshes[i]).collect();
        let group_normals: Vec<&[Vec3]> = idxs.iter().map(|&i| normals[i].as_slice()).collect();
        let compact = compact(&group_meshes, &group_normals);
        if compact.faces.is_empty() {
            continue;
        }
        let mesh_name = if skel_path.is_some() {
            let jl = j.map(|x| x.to_string()).unwrap_or_else(|| "Root".into());
            let ml = if mat_name.is_empty() {
                format!("M{i}")
            } else {
                mat_name.clone()
            };
            format!("Mesh_{jl}_{ml}")
        } else if mat_name.is_empty() {
            format!("Mesh_{i}")
        } else {
            mat_name.clone()
        };
        let mesh_path = format!("{parent}/{mesh_name}");
        mesh::write(&stage, &mesh_path, &compact)?;
        if !mat_name.is_empty() {
            let data = materials.iter().find(|m| clean_name(&m.name) == *mat_name);
            material::define_and_bind(&stage, &root, mat_name, data, &mesh_path)?;
        }
        if let (Some(sp), Some(ji)) = (&skel_path, j) {
            usd_skel::bind(&stage, &mesh_path, sp, *ji, compact.points.len())?;
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
