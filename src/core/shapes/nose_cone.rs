//! NoseCone1 / FairingNoseCone1：多环鼻锥。
//!
//! subdivisions+1 个环，半径按 t^2.1 缓动插值，
//! offset/形变/圆角/clamp 随 t 线性插值；尖端退化为单顶点。

use super::base::Vec3;
use super::section::{apply_clamp, rounded_rect_point, shear_y, shrink_radial};
use super::{Part, RawMesh, Rings};
use std::f64::consts::PI;

pub fn generate(part: &Part, segments: usize) -> Option<RawMesh> {
    let p = part.fuselage.as_ref()?;
    let half_len = p.length / 2.0;
    let [sx, sy, sz] = p.part_scale;
    let has_scale = p.part_scale.iter().any(|v| (v - 1.0).abs() > 1e-6);
    let angle_step = 2.0 * PI / segments as f64;

    let subdivisions = part.subdivisions;
    let num_rings = subdivisions + 1;

    let top_corners = &p.corner_radiuses[..4];
    let bottom_corners = &p.corner_radiuses[4..];
    let [top_deform, shear, bottom_deform] = p.deformations;
    let top_clamp = &p.clamp_distances[..4];
    let bottom_clamp = &p.clamp_distances[4..];

    let wall = part.wall_thickness;
    let hollow = wall > 0.0;

    let mut mesh = RawMesh::new(part);
    let mut rings: Vec<Vec<usize>> = Vec::with_capacity(num_rings);
    let mut inner_rings: Option<Vec<Vec<usize>>> = hollow.then(|| Vec::with_capacity(num_rings));

    for ring in 0..num_rings {
        let t = ring as f64 / subdivisions as f64;
        let y = -half_len + t * p.length;

        // 半径缓动插值；t≈1 收缩为尖端
        let (ratio_x, ratio_z) = if t < 0.99 {
            let ease = t.powf(2.1);
            (
                p.bottom_scale[0] * (1.0 - ease) + p.top_scale[0] * ease,
                p.bottom_scale[1] * (1.0 - ease) + p.top_scale[1] * ease,
            )
        } else {
            (0.0, 0.0)
        };

        let off_x = p.offset[0] * t;
        let off_z = p.offset[2] * t;
        let deform = bottom_deform * (1.0 - t) + top_deform * t;
        let corners: Vec<f64> = (0..4)
            .map(|i| bottom_corners[i] * (1.0 - t) + top_corners[i] * t)
            .collect();
        let clamp: Vec<f64> = (0..4)
            .map(|i| bottom_clamp[i] * (1.0 - t) + top_clamp[i] * t)
            .collect();

        let rz_scaled = p.radius_z * ratio_z * sz;
        let cut_depth = 2.0 * p.offset[1] * shear * t;

        let mut coords: Vec<Vec3> = (0..segments)
            .map(|i| {
                let angle = i as f64 * angle_step;
                let (x, z) = rounded_rect_point(angle, p.radius_x, p.radius_z, &corners, deform);
                [x * ratio_x + off_x, y, z * ratio_z + off_z]
            })
            .collect();
        apply_clamp(&mut coords, &clamp);
        if has_scale {
            for c in coords.iter_mut() {
                c[0] *= sx;
                c[1] *= sy;
                c[2] *= sz;
            }
        }

        let is_tip = ratio_x < 1e-6 && ratio_z < 1e-6;
        let uv_u = |i: usize| i as f64 / segments as f64;

        if is_tip {
            // 尖端：单顶点，整环共享
            let mut c = coords[0];
            c[1] = shear_y(c[1], c[2], rz_scaled, cut_depth);
            let idx = mesh.push_vertex(c, [0.5, t]);
            rings.push(vec![idx; segments]);
            if let Some(ir) = &mut inner_rings {
                ir.push(vec![idx; segments]);
            }
        } else {
            // 内环从 clamp+scale 后的坐标收缩（与圆柱不同，忠实保留旧语义）
            let inner_coords: Option<Vec<Vec3>> = hollow.then(|| {
                coords
                    .iter()
                    .map(|&[x, yy, z]| shrink_radial(x, z, yy, wall))
                    .collect()
            });

            let mut ring_idx = Vec::with_capacity(segments);
            let mut inner_idx = hollow.then(|| Vec::with_capacity(segments));
            for (i, &c) in coords.iter().enumerate() {
                let mut w = c;
                w[1] = shear_y(w[1], w[2], rz_scaled, cut_depth);
                ring_idx.push(mesh.push_vertex(w, [uv_u(i), t]));

                if let (Some(ii), Some(ic)) = (&mut inner_idx, &inner_coords) {
                    let mut wi = ic[i];
                    wi[1] = shear_y(wi[1], wi[2], rz_scaled, cut_depth);
                    ii.push(mesh.push_vertex(wi, [uv_u(i), t]));
                }
            }
            rings.push(ring_idx);
            if let (Some(ir), Some(ii)) = (&mut inner_rings, inner_idx) {
                ir.push(ii);
            }
        }
    }

    // 外侧面
    let next = |i: usize| (i + 1) % segments;
    for ring in 0..subdivisions {
        let cur = &rings[ring];
        let nxt = &rings[ring + 1];
        for i in 0..segments {
            let n = next(i);
            mesh.faces.push([cur[i], nxt[i], nxt[n]]);
            mesh.faces.push([cur[i], nxt[n], cur[n]]);
        }
    }

    // 内侧面（绕序反转）与底部端盖
    let has_bottom_cap = p.bottom_scale[0] > 1e-6 || p.bottom_scale[1] > 1e-6;
    if let Some(ir) = &inner_rings {
        for ring in 0..subdivisions {
            let cur = &ir[ring];
            let nxt = &ir[ring + 1];
            for i in 0..segments {
                let n = next(i);
                mesh.faces.push([cur[i], cur[n], nxt[n]]);
                mesh.faces.push([cur[i], nxt[n], nxt[i]]);
            }
        }
        if has_bottom_cap {
            let br = &rings[0];
            let bi = &ir[0];
            for i in 0..segments {
                let n = next(i);
                mesh.faces.push([br[i], br[n], bi[n]]);
                mesh.faces.push([br[i], bi[n], bi[i]]);
            }
        }
    } else if has_bottom_cap {
        let center = [-p.offset[0] * sx, -half_len * sy, -p.offset[2] * sz];
        let c = mesh.push_vertex(center, [0.5, 0.5]);
        let br = &rings[0];
        for i in 0..segments {
            mesh.faces.push([c, br[i], br[next(i)]]);
        }
    }

    mesh.rings = Rings::NoseCone {
        rings,
        inner_rings,
        has_bottom_cap,
    };
    Some(mesh)
}
