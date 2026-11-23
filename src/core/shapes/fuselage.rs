//! Fuselage1 / Strut1：实心椭圆（圆角矩形）截面圆柱。
//!
//! 共享圆柱几何同时服务空心件（wall_thickness > 0 时生成内环），
//! 供 inlet 复用；法线策略声明后续在此模块归属。

use super::base::Vec3;
use super::section::{apply_clamp, rounded_rect_point, shear_y, shrink_radial};
use super::{Part, PartType, RawMesh, Rings, parse_floats, parse_vec3};
use crate::settings::Settings;
use roxmltree::Node;
use std::f64::consts::PI;

/// Fuselage 参数（语义与 SR2 XML 一一对应）
#[derive(Debug, Clone)]
pub struct FuselageParams {
    /// 长度 = offset_y × 2
    pub length: f64,
    pub radius_x: f64,
    pub radius_z: f64,
    pub offset: Vec3,
    /// (x, z)
    pub top_scale: [f64; 2],
    pub bottom_scale: [f64; 2],
    /// (顶部水滴, 竖向剪切, 底部水滴)
    pub deformations: [f64; 3],
    /// 顶4 + 底4，象限序：左上/右上/右下/左下
    pub corner_radiuses: [f64; 8],
    /// tx-,tx+,tz-,tz+,bx-,bx+,bz-,bz+
    pub clamp_distances: [f64; 8],
    /// 整体额外缩放，旋转前应用
    pub part_scale: Vec3,
}

/// 解析 <Fuselage> 子元素与 Config/partScale
pub fn parse_params(
    elem: Node,
    settings: &Settings,
    part_type: PartType,
) -> Option<FuselageParams> {
    let fus = elem.children().find(|n| n.has_tag_name("Fuselage"))?;

    let mut p = FuselageParams {
        length: 5.0,
        radius_x: settings.fuselage.default_radius_x,
        radius_z: settings.fuselage.default_radius_z,
        offset: [0.0; 3],
        top_scale: [1.0; 2],
        bottom_scale: [1.0; 2],
        deformations: [0.0; 3],
        corner_radiuses: [1.0; 8],
        clamp_distances: [1.0; 8],
        part_scale: [1.0; 3],
    };

    if let Some(offset) = fus.attribute("offset").and_then(parse_vec3) {
        p.offset = offset;
    }
    if let Some(s) = fus.attribute("topScale").map(parse_floats)
        && s.len() >= 2
    {
        p.top_scale = [s[0], s[1]];
    }
    if let Some(s) = fus.attribute("bottomScale").map(parse_floats)
        && s.len() >= 2
    {
        p.bottom_scale = [s[0], s[1]];
    }
    if let Some(d) = fus.attribute("deformations").and_then(parse_vec3) {
        p.deformations = d;
    }
    if let Some(r) = fus.attribute("cornerRadiuses").map(parse_floats)
        && r.len() >= 8
    {
        p.corner_radiuses.copy_from_slice(&r[..8]);
    }
    if let Some(c) = fus.attribute("clampDistances").map(parse_floats)
        && c.len() >= 8
    {
        p.clamp_distances.copy_from_slice(&c[..8]);
    }

    p.length = p.offset[1] * 2.0;

    if let Some(config) = elem.children().find(|n| n.has_tag_name("Config"))
        && let Some(s) = config.attribute("partScale").and_then(parse_vec3)
    {
        p.part_scale = s;
    }

    // Strut1 缺省圆角
    if part_type == PartType::Strut && fus.attribute("cornerRadiuses").is_none() {
        p.corner_radiuses = [settings.fuselage.strut_corner_radius; 8];
    }

    Some(p)
}

/// 生成圆柱网格（局部空间）；wall_thickness > 0 时生成空心内环与环形端盖
pub fn generate(part: &Part, segments: usize) -> Option<RawMesh> {
    let p = part.fuselage.as_ref()?;
    let half_len = p.length / 2.0;
    let [sx, sy, sz] = p.part_scale;
    let has_scale = p.part_scale.iter().any(|v| (v - 1.0).abs() > 1e-6);
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
    if has_scale {
        for ring in [&mut bottom, &mut top] {
            scale_ring(ring, sx, sy, sz);
        }
        if let Some(inner) = &mut bottom_inner {
            scale_ring(inner, sx, sy, sz);
        }
        if let Some(inner) = &mut top_inner {
            scale_ring(inner, sx, sy, sz);
        }
    }

    let mut mesh = RawMesh::new(part);
    let rz_top_scaled = p.radius_z * p.top_scale[1];
    let cut_depth = 2.0 * p.offset[1] * shear;
    let uv_u = |i: usize| i as f64 / segments as f64;

    // 底环顶点（点退化时共享单中心顶点）
    let bottom_idx: Vec<usize> = if bottom_is_point {
        let c = mesh.push_vertex(bottom[0], [0.5, 0.5]);
        vec![c; segments]
    } else {
        bottom
            .iter()
            .enumerate()
            .map(|(i, &v)| mesh.push_vertex(v, [uv_u(i), 0.0]))
            .collect()
    };

    // 顶环顶点（应用竖向剪切）
    let top_idx: Vec<usize> = if top_is_point {
        let mut c = top[0];
        c[1] = shear_y(c[1], c[2], rz_top_scaled, cut_depth);
        let idx = mesh.push_vertex(c, [0.5, 0.5]);
        vec![idx; segments]
    } else {
        top.iter()
            .enumerate()
            .map(|(i, &v)| {
                let mut w = v;
                w[1] = shear_y(w[1], w[2], rz_top_scaled, cut_depth);
                mesh.push_vertex(w, [uv_u(i), 1.0])
            })
            .collect()
    };

    // 内环顶点
    let bottom_inner_idx = bottom_inner.map(|inner| {
        if bottom_is_point {
            vec![bottom_idx[0]; segments]
        } else {
            inner
                .iter()
                .enumerate()
                .map(|(i, &v)| mesh.push_vertex(v, [uv_u(i), 0.0]))
                .collect()
        }
    });
    let top_inner_idx = top_inner.map(|inner| {
        if top_is_point {
            vec![top_idx[0]; segments]
        } else {
            inner
                .iter()
                .enumerate()
                .map(|(i, &v)| {
                    let mut w = v;
                    w[1] = shear_y(w[1], w[2], rz_top_scaled, cut_depth);
                    mesh.push_vertex(w, [uv_u(i), 1.0])
                })
                .collect()
        }
    });

    // 外侧面
    let next = |i: usize| (i + 1) % segments;
    if bottom_is_point && top_is_point {
        // 两面都是点：退化，无侧面
    } else if bottom_is_point {
        for i in 0..segments {
            mesh.faces
                .push([bottom_idx[i], top_idx[i], top_idx[next(i)]]);
        }
    } else if top_is_point {
        for i in 0..segments {
            mesh.faces
                .push([bottom_idx[i], top_idx[i], bottom_idx[next(i)]]);
        }
    } else {
        for i in 0..segments {
            let n = next(i);
            mesh.faces.push([bottom_idx[i], top_idx[i], top_idx[n]]);
            mesh.faces.push([bottom_idx[i], top_idx[n], bottom_idx[n]]);
        }
    }

    if let (Some(bi), Some(ti)) = (bottom_inner_idx, top_inner_idx) {
        // 内侧面（绕序反转）
        if bottom_is_point && top_is_point {
        } else if bottom_is_point {
            for i in 0..segments {
                mesh.faces.push([bi[i], ti[next(i)], ti[i]]);
            }
        } else if top_is_point {
            for i in 0..segments {
                mesh.faces.push([bi[i], bi[next(i)], ti[i]]);
            }
        } else {
            for i in 0..segments {
                let n = next(i);
                mesh.faces.push([bi[i], ti[n], ti[i]]);
                mesh.faces.push([bi[i], bi[n], ti[n]]);
            }
        }
        // 环形端盖
        if !bottom_is_point {
            for i in 0..segments {
                let n = next(i);
                mesh.faces.push([bottom_idx[i], bottom_idx[n], bi[n]]);
                mesh.faces.push([bottom_idx[i], bi[n], bi[i]]);
            }
        }
        if !top_is_point {
            for i in 0..segments {
                let n = next(i);
                mesh.faces.push([top_idx[i], ti[i], ti[n]]);
                mesh.faces.push([top_idx[i], ti[n], top_idx[n]]);
            }
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
        // 实心端盖
        if !bottom_is_point {
            let center = [-p.offset[0] * sx, -half_len * sy, -p.offset[2] * sz];
            let c = mesh.push_vertex(center, [0.5, 0.5]);
            for i in 0..segments {
                mesh.faces.push([c, bottom_idx[i], bottom_idx[next(i)]]);
            }
        }
        if !top_is_point {
            let cd = 2.0 * p.offset[1] * shear * sy;
            let center = [p.offset[0] * sx, half_len * sy - cd / 2.0, p.offset[2] * sz];
            let c = mesh.push_vertex(center, [0.5, 0.5]);
            for i in 0..segments {
                mesh.faces.push([c, top_idx[next(i)], top_idx[i]]);
            }
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

fn scale_ring(ring: &mut [Vec3], sx: f64, sy: f64, sz: f64) {
    for c in ring.iter_mut() {
        c[0] *= sx;
        c[1] *= sy;
        c[2] *= sz;
    }
}
