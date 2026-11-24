//! Fuselage1 / Strut1 几何：实心椭圆（圆角矩形）截面圆柱。
//!
//! 共享圆柱几何同时服务空心件（wall_thickness > 0 时生成内环），
//! 供 inlet 复用；法线策略声明后续在此模块归属。

use super::super::algo::ring::{cap_annulus, cap_fan, emit_ring, stitch_side};
use super::super::algo::section::{apply_clamp, rounded_rect_point, shear_y, shrink_radial};
use super::super::algo::transform::{Vec3, scale_ring};
use super::model::{FaceGroup, RawMesh, Rim};
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

    // 空心尖点内缩：内壁半径=外半径-wall，归零处比外尖点靠内 recess=wall*长/底半径，保留壁厚
    let r_eff_b = (p.radius_x * p.bottom_scale[0]).max(p.radius_z * p.bottom_scale[1]);
    let r_eff_t = (p.radius_x * p.top_scale[0]).max(p.radius_z * p.top_scale[1]);
    let recess_t = if r_eff_b > 1e-9 {
        (wall * p.length / r_eff_b).min(p.length)
    } else {
        0.0
    };
    let recess_b = if r_eff_t > 1e-9 {
        (wall * p.length / r_eff_t).min(p.length)
    } else {
        0.0
    };

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
                [0.0, -half_len + recess_b, 0.0]
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
                [0.0, half_len - recess_t, 0.0]
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

    // 发射环顶点（点退化共享中心顶点；内外环各自独立中心，不合并）
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
            emit_ring(&mut mesh, &inner, 0.0, Some([0.5, 0.5]))
        } else {
            emit_ring(&mut mesh, &inner, 0.0, None)
        }
    });
    let top_inner_idx = top_inner.map(|inner| {
        if top_is_point {
            emit_ring(&mut mesh, &inner, 1.0, Some([0.5, 0.5]))
        } else {
            emit_ring(&mut mesh, &inner, 1.0, None)
        }
    });

    // 缩 0 端：内外中心各自硬点（锥面母线保持光滑，避免整面放射状硬棱）
    if bottom_is_point {
        mesh.hard_vertices.insert(bottom_idx[0]);
        if let Some(bi) = &bottom_inner_idx {
            mesh.hard_vertices.insert(bi[0]);
        }
    }
    if top_is_point {
        mesh.hard_vertices.insert(top_idx[0]);
        if let Some(ti) = &top_inner_idx {
            mesh.hard_vertices.insert(ti[0]);
        }
    }

    // 侧面与端盖（侧面保留四边面，端盖三角；侧面/端盖分 pass 算法线）
    stitch_side(
        &mut mesh,
        &bottom_idx,
        &top_idx,
        bottom_is_point,
        top_is_point,
        false,
    );

    if let (Some(bi), Some(ti)) = (bottom_inner_idx, top_inner_idx) {
        stitch_side(&mut mesh, &bi, &ti, bottom_is_point, top_is_point, true);
        if !bottom_is_point {
            let mut f = Vec::new();
            cap_annulus(&mut f, &bottom_idx, &bi, false);
            mesh.add_faces(f, FaceGroup::Cap);
        }
        if !top_is_point {
            let mut f = Vec::new();
            cap_annulus(&mut f, &top_idx, &ti, true);
            mesh.add_faces(f, FaceGroup::Cap);
        }
    } else {
        let [sx, sy, sz] = p.part_scale;
        if !bottom_is_point {
            let center = [-p.offset[0] * sx, -half_len * sy, -p.offset[2] * sz];
            let c = mesh.push_vertex(center, [0.5, 0.5]);
            let mut f = Vec::new();
            cap_fan(&mut f, c, &bottom_idx, false);
            mesh.add_faces(f, FaceGroup::Cap);
        }
        if !top_is_point {
            let cd = 2.0 * p.offset[1] * shear * sy;
            let center = [p.offset[0] * sx, half_len * sy - cd / 2.0, p.offset[2] * sz];
            let c = mesh.push_vertex(center, [0.5, 0.5]);
            let mut f = Vec::new();
            cap_fan(&mut f, c, &top_idx, true);
            mesh.add_faces(f, FaceGroup::Cap);
        }
    }

    // 四边面优化三角化（法线计算前）
    super::super::algo::triangulate::triangulate(&mut mesh);

    // 方形截面：所有侧面棱强制锐边（共面平面法线相同仍显平，90° 角保持硬）
    if p.corner_radiuses.iter().all(|&r| r <= 0.001) {
        for (fi, f) in mesh.faces.iter().enumerate() {
            if mesh.face_groups[fi] != FaceGroup::Side {
                continue;
            }
            for k in 0..3 {
                let a = f[k];
                let b = f[(k + 1) % 3];
                mesh.sharp_edges.insert((a.min(b), a.max(b)));
            }
        }
    }

    // 端环：供接头法线混合（点退化端无环；smooth=该端开平滑）
    if !bottom_is_point {
        mesh.rims.push(Rim {
            vertices: bottom_idx.clone(),
            smooth: p.flatten_normals.smooth_bottom(),
        });
    }
    if !top_is_point {
        mesh.rims.push(Rim {
            vertices: top_idx.clone(),
            smooth: p.flatten_normals.smooth_top(),
        });
    }

    Some(mesh)
}

#[cfg(test)]
#[path = "../../../../tests/src/core/shapes/components/fuselage_test.rs"]
mod tests;
