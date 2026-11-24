//! 法线编排：侧面跑 Blender 自动平滑，端盖单独算平面法线，输出 faceVarying 法线。
//!
//! 侧面/端盖分 pass：端盖硬边独立计算，避免与侧面相互拆边。
//! 世界变换由调用方经 algo::transform 完成（或直接在合并后的世界网格上计算）。

use super::smooth;
use crate::core::shapes::RawMesh;
use crate::core::shapes::algo::transform::{Vec3, dot, face_normal};
use crate::core::shapes::components::model::FaceGroup;

/// 输出长度 = 3 × 面数，每面三个顶点法线按序排列
pub fn compute(mesh: &RawMesh, sharp_edge_angle: f64) -> Vec<Vec3> {
    let mut out = vec![[0.0; 3]; mesh.faces.len() * 3];

    let mut side: Vec<usize> = Vec::new();
    let mut cap: Vec<usize> = Vec::new();
    for (fi, g) in mesh.face_groups.iter().enumerate() {
        match g {
            FaceGroup::Side => side.push(fi),
            FaceGroup::Cap => cap.push(fi),
        }
    }

    // 侧面：自动平滑
    let side_faces: Vec<[usize; 3]> = side.iter().map(|&i| mesh.faces[i]).collect();
    // 强制锐边：零件声明（方形棱）+ 尖点入射边（圆锥顶点 / 缩 0 端保持尖锐）
    let mut sharp_edges: std::collections::HashSet<(usize, usize)> =
        std::collections::HashSet::new();
    sharp_edges.extend(mesh.sharp_edges.iter().copied());
    if !mesh.apex_vertices.is_empty() {
        for f in &side_faces {
            for k in 0..3 {
                let a = f[k];
                let b = f[(k + 1) % 3];
                if mesh.apex_vertices.contains(&a) || mesh.apex_vertices.contains(&b) {
                    sharp_edges.insert((a.min(b), a.max(b)));
                }
            }
        }
    }
    let ns = smooth::smooth_normals(
        &mesh.vertices,
        &side_faces,
        sharp_edge_angle,
        &mesh.smooth_edges,
        &sharp_edges,
        &mesh.hard_vertices,
    );
    for (k, &fi) in side.iter().enumerate() {
        out[fi * 3..fi * 3 + 3].copy_from_slice(&ns[k * 3..k * 3 + 3]);
    }

    // 端盖：平面法线（朝向由绕序决定），整面共享→硬边
    for &fi in &cap {
        let f = mesh.faces[fi];
        let n = face_normal(&mesh.vertices, &f);
        let n = if dot(n, n) > 0.5 { n } else { [0.0, 1.0, 0.0] };
        for k in 0..3 {
            out[fi * 3 + k] = n;
        }
    }

    out
}

#[cfg(test)]
#[path = "../../../tests/src/core/normal/compute_test.rs"]
mod tests;
