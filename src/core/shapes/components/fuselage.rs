//! Fuselage1 / Strut1 几何：实心椭圆（圆角矩形）截面圆柱。
//!
//! 共享圆柱几何同时服务空心件（wall_thickness > 0 时生成内环），
//! 供 inlet 复用；法线策略声明后续在此模块归属。

use super::super::algo::ring::{cap_annulus, cap_fan, emit_ring, stitch_side};
use super::super::algo::section::{apply_clamp, rounded_rect_point, shear_y, shrink_radial};
use super::super::algo::transform::{Vec3, scale_ring};
use super::model::{RawMesh, Rings};
use crate::core::parse::Part;
use std::f64::consts::PI;

/// 生成圆柱网格（局部空间）；wall_thickness > 0 时生成空心内环与环形端盖
pub fn generate(part: &Part, segments: usize) -> Option<RawMesh> {
    let p = part.fuselage.as_ref()?;
    let half_len = p.length / 2.0;
    let angle_step = 2.0 * PI / segments as f64;

    let top_corners = &p.corner_radiuses[..4];
    let bottom_corners = &p.corner_radiuses[4..];
    let [top_deform, shear, bottom_deform] = p.deformations;
    let top_clamp = &p.clamp_distances[..4];
    let bottom_clamp = &p.clamp_distances[4..];

    let wall = part.wall_thickness;
    let hollow = wall > 0.0;
    let bottom_is_point = p.bottom_scale[0] < 1e-6 && p.bottom_scale[1] < 1e-6;
    let top_is_point = p.top_scale[0] < 1e-6 && p.top_scale[1] < 1e-6;

    // 原始环坐标
    let mut bottom = Vec::with_capacity(segments);
    let mut top = Vec::with_capacity(segments);
    let mut bottom_inner = hollow.then(|| Vec::with_capacity(segments));
    let mut top_inner = hollow.then(|| Vec::with_capacity(segments));

    for i in 0..segments {
        let angle = i as f64 * angle_step;

        let (bx, bz) = if bottom_is_point {
            (-p.offset[0], -p.offset[2])
        } else {
            let (x, z) =
                rounded_rect_point(angle, p.radius_x, p.radius_z, bottom_corners, bottom_deform);
            (
                x * p.bottom_scale[0] - p.offset[0],
                z * p.bottom_scale[1] - p.offset[2],
            )
        };
        bottom.push([bx, -half_len, bz]);
        if let Some(inner) = &mut bottom_inner {
            inner.push(if bottom_is_point {
                [0.0, -half_len, 0.0]
            } else {
                shrink_radial(bx, bz, -half_len, wall)
            });
        }

        let (tx, tz) = if top_is_point {
            (p.offset[0], p.offset[2])
        } else {
            let (x, z) = rounded_rect_point(angle, p.radius_x, p.radius_z, top_corners, top_deform);
            (
                x * p.top_scale[0] + p.offset[0],
                z * p.top_scale[1] + p.offset[2],
            )
        };
        top.push([tx, half_len, tz]);
        if let Some(inner) = &mut top_inner {
            inner.push(if top_is_point {
                [0.0, half_len, 0.0]
            } else {
                shrink_radial(tx, tz, half_len, wall)
            });
        }
    }

    // clamp 与 partScale
    apply_clamp(&mut bottom, bottom_clamp);
    apply_clamp(&mut top, top_clamp);
    if let Some(inner) = &mut bottom_inner {
        apply_clamp(inner, bottom_clamp);
    }
    if let Some(inner) = &mut top_inner {
        apply_clamp(inner, top_clamp);
    }
    let has_scale = p.part_scale.iter().any(|v| (v - 1.0).abs() > 1e-6);
    if has_scale {
        scale_ring(&mut bottom, p.part_scale);
        scale_ring(&mut top, p.part_scale);
        if let Some(inner) = &mut bottom_inner {
            scale_ring(inner, p.part_scale);
        }
        if let Some(inner) = &mut top_inner {
            scale_ring(inner, p.part_scale);
        }
    }

    // 顶环竖向剪切
    let rz_top_scaled = p.radius_z * p.top_scale[1];
    let cut_depth = 2.0 * p.offset[1] * shear;
    let shear_ring = |ring: &[Vec3]| -> Vec<Vec3> {
        ring.iter()
            .map(|&c| [c[0], shear_y(c[1], c[2], rz_top_scaled, cut_depth), c[2]])
            .collect()
    };
    let top = shear_ring(&top);
    let top_inner = top_inner.map(|inner| shear_ring(&inner));

    // 发射环顶点（点退化共享中心顶点；内环点退化复用外环中心）
    let mut mesh = RawMesh::new(part);
    let bottom_idx = emit_ring(
        &mut mesh,
        &bottom,
        0.0,
        bottom_is_point.then_some([0.5, 0.5]),
    );
    let top_idx = emit_ring(&mut mesh, &top, 1.0, top_is_point.then_some([0.5, 0.5]));
    let bottom_inner_idx = bottom_inner.map(|inner| {
        if bottom_is_point {
            vec![bottom_idx[0]; segments]
        } else {
            emit_ring(&mut mesh, &inner, 0.0, None)
        }
    });
    let top_inner_idx = top_inner.map(|inner| {
        if top_is_point {
            vec![top_idx[0]; segments]
        } else {
            emit_ring(&mut mesh, &inner, 1.0, None)
        }
    });

    // 侧面与端盖
    stitch_side(
        &mut mesh.faces,
        &bottom_idx,
        &top_idx,
        bottom_is_point,
        top_is_point,
        false,
    );
    if let (Some(bi), Some(ti)) = (bottom_inner_idx, top_inner_idx) {
        stitch_side(
            &mut mesh.faces,
            &bi,
            &ti,
            bottom_is_point,
            top_is_point,
            true,
        );
        if !bottom_is_point {
            cap_annulus(&mut mesh.faces, &bottom_idx, &bi, false);
        }
        if !top_is_point {
            cap_annulus(&mut mesh.faces, &top_idx, &ti, true);
        }
        mesh.rings = Rings::Cylinder {
            bottom: bottom_idx,
            top: top_idx,
            bottom_inner: Some(bi),
            top_inner: Some(ti),
            bottom_corners: bottom_corners.try_into().unwrap(),
            top_corners: top_corners.try_into().unwrap(),
            bottom_is_point,
            top_is_point,
        };
    } else {
        let [sx, sy, sz] = p.part_scale;
        if !bottom_is_point {
            let center = [-p.offset[0] * sx, -half_len * sy, -p.offset[2] * sz];
            let c = mesh.push_vertex(center, [0.5, 0.5]);
            cap_fan(&mut mesh.faces, c, &bottom_idx, false);
        }
        if !top_is_point {
            let cd = 2.0 * p.offset[1] * shear * sy;
            let center = [p.offset[0] * sx, half_len * sy - cd / 2.0, p.offset[2] * sz];
            let c = mesh.push_vertex(center, [0.5, 0.5]);
            cap_fan(&mut mesh.faces, c, &top_idx, true);
        }
        mesh.rings = Rings::Cylinder {
            bottom: bottom_idx,
            top: top_idx,
            bottom_inner: None,
            top_inner: None,
            bottom_corners: bottom_corners.try_into().unwrap(),
            top_corners: top_corners.try_into().unwrap(),
            bottom_is_point,
            top_is_point,
        };
    }

    Some(mesh)
}
