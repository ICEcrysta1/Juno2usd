//! USD 导出编排：按材质分组，按位置焊接顶点后在合并网格上计算法线
//! （零件对接处共享法线→连接光滑），再紧凑化写出；不做几何生成。

use super::mesh::CompactMesh;
use super::{material, mesh, stage};
use crate::core::normal;
use crate::core::parse::material::{Material, clean_name};
use crate::core::shapes::RawMesh;
use crate::core::shapes::algo::transform::Vec3;
use crate::core::shapes::components::model::FaceGroup;
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// 材质分组：(材质名, [网格])
type MaterialGroup<'a> = (String, Vec<&'a RawMesh>);

/// 导出 USDA：根 Xform + 每材质一个 Mesh + UsdPreviewSurface + faceVarying 法线
pub fn export(
    output: &Path,
    prefix: &str,
    materials: &[Material],
    meshes: &[RawMesh],
    sharp_angle: f64,
) -> Result<(), String> {
    let (stage, root) = stage::create(prefix)?;

    // 按材质分组（保持首次出现顺序）
    let mut groups: Vec<MaterialGroup> = Vec::new();
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
        // 焊接代理网格：重合顶点合并，法线在合并网格上计算→零件间光滑
        let (proxy, face_offset) = weld(group);
        let ns = normal::compute(&proxy, sharp_angle);

        let compact = compact(group, &face_offset, &ns);
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

/// 按位置焊接顶点（量化 1e-5），合并面/分组/锐边；
/// 返回代理网格与各原始网格的起始面序号（用于法线回写）
fn weld(group: &[&RawMesh]) -> (RawMesh, Vec<usize>) {
    let mut weld_map: HashMap<[i64; 3], usize> = HashMap::new();
    let mut points: Vec<Vec3> = Vec::new();
    let mut faces: Vec<[usize; 3]> = Vec::new();
    let mut face_groups: Vec<FaceGroup> = Vec::new();
    let mut sharp_edges: HashSet<(usize, usize)> = HashSet::new();
    let mut face_offset = Vec::with_capacity(group.len());

    let key = |v: Vec3| {
        [
            (v[0] * 1e5).round() as i64,
            (v[1] * 1e5).round() as i64,
            (v[2] * 1e5).round() as i64,
        ]
    };

    for m in group {
        face_offset.push(faces.len());
        let mut remap: HashMap<usize, usize> = HashMap::new();
        for (vi, &v) in m.vertices.iter().enumerate() {
            let wi = *weld_map.entry(key(v)).or_insert_with(|| {
                points.push(v);
                points.len() - 1
            });
            remap.insert(vi, wi);
        }
        for f in &m.faces {
            faces.push([remap[&f[0]], remap[&f[1]], remap[&f[2]]]);
        }
        face_groups.extend_from_slice(&m.face_groups);
        for &(a, b) in &m.sharp_edges {
            let (a, b) = (remap[&a], remap[&b]);
            sharp_edges.insert((a.min(b), a.max(b)));
        }
    }

    let proxy = RawMesh {
        part_id: "welded".to_string(),
        material_name: String::new(),
        vertices: points,
        uvs: Vec::new(),
        faces,
        face_groups,
        sharp_edges,
    };
    (proxy, face_offset)
}

/// 合并同材质网格：过滤退化面，紧凑化被引用的顶点（保留各自 UV），
/// 法线按面序号从焊接网格的计算结果回写
fn compact(group: &[&RawMesh], face_offset: &[usize], ns: &[Vec3]) -> CompactMesh {
    let mut points: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut faces: Vec<[i32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();

    for (mi, m) in group.iter().enumerate() {
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
            let gi = face_offset[mi] + fi;
            for k in 0..3 {
                normals.push(ns[gi * 3 + k].map(|c| c as f32));
            }
        }
    }

    CompactMesh {
        points,
        uvs,
        faces,
        normals,
    }
}
