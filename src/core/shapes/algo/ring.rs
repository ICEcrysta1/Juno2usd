//! 环拓扑算法：顶点发射、环间侧面缝合、端盖。
//!
//! 绕序约定：invert=false 为正绕序（外侧面、朝下端盖），
//! invert=true 反转（内侧面、朝上端盖）；
//! 环退化为点时共享单中心顶点，侧面退化为三角扇。

use super::super::RawMesh;
use super::transform::Vec3;

/// 环 perimeter 边（无序顶点对）
pub fn perimeter_edges(ring: &[usize]) -> Vec<(usize, usize)> {
    let n = ring.len();
    (0..n)
        .map(|i| {
            let a = ring[i];
            let b = ring[(i + 1) % n];
            (a.min(b), a.max(b))
        })
        .collect()
}

/// 发射环顶点（UV 的 v 坐标固定）；point_uv 为 Some 时退化为单中心顶点，整环共享
pub fn emit_ring(
    mesh: &mut RawMesh,
    coords: &[Vec3],
    uv_v: f64,
    point_uv: Option<[f64; 2]>,
) -> Vec<usize> {
    let n = coords.len();
    if let Some(uv) = point_uv {
        let c = mesh.push_vertex(coords[0], uv);
        return vec![c; n];
    }
    coords
        .iter()
        .enumerate()
        .map(|(i, &p)| mesh.push_vertex(p, [i as f64 / n as f64, uv_v]))
        .collect()
}

fn push_tri(faces: &mut Vec<[usize; 3]>, a: usize, b: usize, c: usize, invert: bool) {
    faces.push(if invert { [a, c, b] } else { [a, b, c] });
}

/// 两环之间的侧面；环为点时从中心发射三角扇
pub fn stitch_side(
    faces: &mut Vec<[usize; 3]>,
    lower: &[usize],
    upper: &[usize],
    lower_point: bool,
    upper_point: bool,
    invert: bool,
) {
    let n = lower.len();
    if lower_point && upper_point {
        return;
    }
    for i in 0..n {
        let j = (i + 1) % n;
        if lower_point {
            push_tri(faces, lower[i], upper[i], upper[j], invert);
        } else if upper_point {
            push_tri(faces, lower[i], upper[i], lower[j], invert);
        } else {
            push_tri(faces, lower[i], upper[i], upper[j], invert);
            push_tri(faces, lower[i], upper[j], lower[j], invert);
        }
    }
}

/// 实心端盖：中心顶点 + 三角扇
pub fn cap_fan(faces: &mut Vec<[usize; 3]>, center: usize, ring: &[usize], invert: bool) {
    let n = ring.len();
    for i in 0..n {
        let j = (i + 1) % n;
        push_tri(faces, center, ring[i], ring[j], invert);
    }
}

/// 空心环形端盖
pub fn cap_annulus(faces: &mut Vec<[usize; 3]>, outer: &[usize], inner: &[usize], invert: bool) {
    let n = outer.len();
    for i in 0..n {
        let j = (i + 1) % n;
        push_tri(faces, outer[i], outer[j], inner[j], invert);
        push_tri(faces, outer[i], inner[j], inner[i], invert);
    }
}
