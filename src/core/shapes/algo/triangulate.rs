//! 四边面优化三角化（法线计算前执行）。
//!
//! 生成期侧面保留四边面；此处统一剖分：
//! - 塌缩四边形（<4 个不同顶点）→ 单个三角扇，避免狭长 sliver/阶梯；
//! - 完整四边形 → 选两三角法线偏离_quad 法线_最小的对角线（非平面时减少折痕），
//!   偏差相同选较短对角线（更均衡的三角形）。

use super::super::RawMesh;
use super::transform::{Vec3, dist, dot, face_normal, ring_normal};

/// 把 mesh.quads 剖分为三角面转入 mesh.faces（保持绕序）
pub fn triangulate(mesh: &mut RawMesh) {
    let quads = std::mem::take(&mut mesh.quads);
    let groups = std::mem::take(&mut mesh.quad_groups);
    let mut tris: Vec<[usize; 3]> = Vec::new();
    let mut tg = Vec::new();
    for (q, g) in quads.iter().zip(groups) {
        let lp = dedup_consecutive(q);
        match lp.len() {
            3 => {
                tris.push([lp[0], lp[1], lp[2]]);
                tg.push(g);
            }
            4 => {
                let (t1, t2) = best_split(&mesh.vertices, &lp);
                tris.push(t1);
                tris.push(t2);
                tg.push(g);
                tg.push(g);
            }
            _ => {}
        }
    }
    mesh.faces.extend(tris);
    mesh.face_groups.extend(tg);
}

/// 去除环序中连续重复顶点（含首尾），塌缩环退化为三角
fn dedup_consecutive(q: &[usize; 4]) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::new();
    for &v in q {
        if out.last() != Some(&v) {
            out.push(v);
        }
    }
    if out.len() > 1 && out.first() == out.last() {
        out.pop();
    }
    out
}

/// 选最优对角线：最小化两三角法线对 quad 法线的偏差，平局选较短对角线
fn best_split(verts: &[Vec3], lp: &[usize]) -> ([usize; 3], [usize; 3]) {
    let [a, b, c, d] = [lp[0], lp[1], lp[2], lp[3]];
    let qn = ring_normal(verts, lp);
    let dev = |t1: [usize; 3], t2: [usize; 3]| {
        (1.0 - dot(face_normal(verts, &t1), qn)) + (1.0 - dot(face_normal(verts, &t2), qn))
    };
    let d1 = dev([a, b, c], [a, c, d]);
    let d2 = dev([a, b, d], [b, c, d]);
    let pick1 = if (d1 - d2).abs() > 1e-9 {
        d1 < d2
    } else {
        dist(verts[a], verts[c]) <= dist(verts[b], verts[d])
    };
    if pick1 {
        ([a, b, c], [a, c, d])
    } else {
        ([a, b, d], [b, c, d])
    }
}
