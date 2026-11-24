//! 连接感知接头法线混合。
//!
//! 每端开关由 flattenNormals 决定（被覆盖的端 = 开平滑）。对每条连接定位对接环后：
//! - 双开：对接环对应顶点侧面法线互相平均；
//! - 单开：开侧环法线对齐关侧（开侧迁就关侧，关侧保持不变）；
//! - 双关：不平滑。
//!
//! 仅处理端面对接：两环须共轴（环平面法线平行），侧向连接（环平面垂直）跳过。
//! 方形等带硬角的环整体不混合（90° 棱保持硬边），只平滑连续曲面环。
//! 端盖法线不参与（保持硬边）。

use crate::core::parse::connection::ConnectionTree;
use crate::core::shapes::RawMesh;
use crate::core::shapes::algo::transform::{Vec3, cross, dot, normalize};
use crate::core::shapes::components::model::{FaceGroup, Rim};
use std::collections::HashMap;

/// 共轴判据：环平面法线 |点积| 下限（cos60°），低于此视为侧向连接
const COAXIAL_COS: f64 = 0.5;

/// meshes/normals 按零件顺序索引，part_ids 为对应零件 ID。
/// threshold_deg 为硬边阈值：带硬角的环（方形棱）不混合。
pub fn blend_joints(
    meshes: &[RawMesh],
    normals: &mut [Vec<Vec3>],
    part_ids: &[String],
    connections: &ConnectionTree,
    threshold_deg: f64,
) {
    let idx_by_id: HashMap<&str, usize> = part_ids
        .iter()
        .enumerate()
        .map(|(i, id)| (id.as_str(), i))
        .collect();
    let cos_thresh = threshold_deg.to_radians().cos();

    for conn in &connections.connections {
        let (Some(&ia), Some(&ib)) = (
            idx_by_id.get(conn.part_a.as_str()),
            idx_by_id.get(conn.part_b.as_str()),
        ) else {
            continue;
        };
        let Some((ra, rb)) = closest_rims(&meshes[ia], &meshes[ib]) else {
            continue;
        };
        // 双关：不平滑
        if !ra.smooth && !rb.smooth {
            continue;
        }
        // 侧向连接（环平面不共轴）：跳过
        if dot(rim_normal(&meshes[ia], ra), rim_normal(&meshes[ib], rb)).abs() < COAXIAL_COS {
            continue;
        }
        // 带硬角的环（方形截面）：整体不混合，保持 90° 硬边
        if rim_has_hard_corner(&meshes[ia], ra, cos_thresh)
            || rim_has_hard_corner(&meshes[ib], rb, cos_thresh)
        {
            continue;
        }
        blend_rim_pair(meshes, normals, ia, ra, ib, rb);
    }
}

/// 找两零件质心最近的一对端环（即对接环）
fn closest_rims<'a>(a: &'a RawMesh, b: &'a RawMesh) -> Option<(&'a Rim, &'a Rim)> {
    let mut best: Option<(f64, &Rim, &Rim)> = None;
    for ra in &a.rims {
        for rb in &b.rims {
            let d = dist(centroid(a, ra), centroid(b, rb));
            if best.is_none_or(|(bd, _, _)| d < bd) {
                best = Some((d, ra, rb));
            }
        }
    }
    best.map(|(_, ra, rb)| (ra, rb))
}

fn centroid(mesh: &RawMesh, rim: &Rim) -> Vec3 {
    let mut c = [0.0; 3];
    for &v in &rim.vertices {
        let p = mesh.vertices[v];
        c[0] += p[0];
        c[1] += p[1];
        c[2] += p[2];
    }
    let n = rim.vertices.len() as f64;
    [c[0] / n, c[1] / n, c[2] / n]
}

/// 环平面法线（Newell），单位化
fn rim_normal(mesh: &RawMesh, rim: &Rim) -> Vec3 {
    let mut n = [0.0; 3];
    let vs = &rim.vertices;
    for i in 0..vs.len() {
        let a = mesh.vertices[vs[i]];
        let b = mesh.vertices[vs[(i + 1) % vs.len()]];
        n[0] += (a[1] - b[1]) * (a[2] + b[2]);
        n[1] += (a[2] - b[2]) * (a[0] + b[0]);
        n[2] += (a[0] - b[0]) * (a[1] + b[1]);
    }
    normalize(n)
}

fn dist(a: Vec3, b: Vec3) -> f64 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    dot(d, d).sqrt()
}

/// 环上是否存在硬角顶点（入射侧面法线夹角 > 阈值，如方形 90° 棱）
fn rim_has_hard_corner(mesh: &RawMesh, rim: &Rim, cos_thresh: f64) -> bool {
    rim.vertices.iter().any(|&v| {
        let ns: Vec<Vec3> = mesh
            .faces
            .iter()
            .enumerate()
            .filter(|(fi, f)| mesh.face_groups[*fi] == FaceGroup::Side && f.contains(&v))
            .map(|(_, f)| face_normal(mesh, f))
            .collect();
        for i in 0..ns.len() {
            for j in i + 1..ns.len() {
                if dot(ns[i], ns[j]) < cos_thresh {
                    return true;
                }
            }
        }
        false
    })
}

fn face_normal(mesh: &RawMesh, f: &[usize; 3]) -> Vec3 {
    let p1 = mesh.vertices[f[0]];
    let p2 = mesh.vertices[f[1]];
    let p3 = mesh.vertices[f[2]];
    normalize(cross(
        [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]],
        [p3[0] - p1[0], p3[1] - p1[1], p3[2] - p1[2]],
    ))
}

/// 按开关规则混合一对对接环（环已确认无硬角，逐顶点平均/对齐）
fn blend_rim_pair(
    meshes: &[RawMesh],
    normals: &mut [Vec<Vec3>],
    ia: usize,
    ra: &Rim,
    ib: usize,
    rb: &Rim,
) {
    let na: Vec<Vec3> = ra
        .vertices
        .iter()
        .map(|&v| vertex_side_normal(&meshes[ia], &normals[ia], v))
        .collect();
    let nb: Vec<Vec3> = rb
        .vertices
        .iter()
        .map(|&v| vertex_side_normal(&meshes[ib], &normals[ib], v))
        .collect();

    match (ra.smooth, rb.smooth) {
        // 双开：互相平均（先算后写，避免顺序偏差）
        (true, true) => {
            let mut wa: Vec<(usize, Vec3)> = Vec::new();
            let mut wb: Vec<(usize, Vec3)> = Vec::new();
            for (ka, &va) in ra.vertices.iter().enumerate() {
                let kb = nearest(&meshes[ia], va, &meshes[ib], &rb.vertices);
                let n = normalize([
                    na[ka][0] + nb[kb][0],
                    na[ka][1] + nb[kb][1],
                    na[ka][2] + nb[kb][2],
                ]);
                if dot(n, n) > 0.5 {
                    wa.push((va, n));
                    wb.push((rb.vertices[kb], n));
                }
            }
            write_all(&meshes[ia], &mut normals[ia], &wa);
            write_all(&meshes[ib], &mut normals[ib], &wb);
        }
        // A 开 B 关：A 对齐 B
        (true, false) => {
            let mut wa: Vec<(usize, Vec3)> = Vec::new();
            for &va in &ra.vertices {
                let kb = nearest(&meshes[ia], va, &meshes[ib], &rb.vertices);
                if dot(nb[kb], nb[kb]) > 0.5 {
                    wa.push((va, nb[kb]));
                }
            }
            write_all(&meshes[ia], &mut normals[ia], &wa);
        }
        // A 关 B 开：B 对齐 A
        (false, true) => {
            let mut wb: Vec<(usize, Vec3)> = Vec::new();
            for &vb in &rb.vertices {
                let ka = nearest(&meshes[ib], vb, &meshes[ia], &ra.vertices);
                if dot(na[ka], na[ka]) > 0.5 {
                    wb.push((vb, na[ka]));
                }
            }
            write_all(&meshes[ib], &mut normals[ib], &wb);
        }
        (false, false) => {}
    }
}

/// 目标顶点在另一环中的最近邻索引
fn nearest(mesh: &RawMesh, v: usize, other: &RawMesh, ring: &[usize]) -> usize {
    let p = mesh.vertices[v];
    (0..ring.len())
        .min_by(|&x, &y| {
            dist(p, other.vertices[ring[x]]).total_cmp(&dist(p, other.vertices[ring[y]]))
        })
        .unwrap()
}

/// 顶点侧面代表法线：该顶点所有侧面面角法线的平均
fn vertex_side_normal(mesh: &RawMesh, normals: &[Vec3], v: usize) -> Vec3 {
    let mut acc = [0.0; 3];
    let mut count = 0;
    for (fi, f) in mesh.faces.iter().enumerate() {
        if mesh.face_groups[fi] != FaceGroup::Side {
            continue;
        }
        for k in 0..3 {
            if f[k] == v {
                let n = normals[fi * 3 + k];
                acc[0] += n[0];
                acc[1] += n[1];
                acc[2] += n[2];
                count += 1;
            }
        }
    }
    if count > 0 { normalize(acc) } else { [0.0; 3] }
}

/// 把法线写回顶点的所有侧面面角
fn write_all(mesh: &RawMesh, normals: &mut [Vec3], writes: &[(usize, Vec3)]) {
    for &(v, n) in writes {
        for (fi, f) in mesh.faces.iter().enumerate() {
            if mesh.face_groups[fi] != FaceGroup::Side {
                continue;
            }
            for k in 0..3 {
                if f[k] == v {
                    normals[fi * 3 + k] = n;
                }
            }
        }
    }
}
