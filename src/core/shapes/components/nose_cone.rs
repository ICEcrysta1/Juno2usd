//! NoseCone1 / FairingNoseCone1 几何：多环鼻锥。
//!
//! subdivisions+1 个环，半径按 t^2.1 缓动插值，
//! offset/形变/圆角/clamp 随 t 线性插值；尖端退化为单顶点。

use super::super::algo::ring::{cap_annulus, cap_fan, emit_ring, perimeter_edges, stitch_side};
use super::super::algo::section::{apply_clamp, rounded_rect_point, shear_y, shrink_radial};
use super::super::algo::transform::{Vec3, scale_ring};
use super::model::{FaceGroup, RawMesh, Rim};
use crate::core::parse::Part;
use std::f64::consts::PI;

pub fn generate(part: &Part, segments: usize) -> Option<RawMesh> {
    let p = part.fuselage.as_ref()?;
    let half_len = p.length / 2.0;
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
    let has_scale = p.part_scale.iter().any(|v| (v - 1.0).abs() > 1e-6);

    let mut mesh = RawMesh::new(part);
    let mut rings: Vec<Vec<usize>> = Vec::with_capacity(num_rings);
    let mut tip_flags: Vec<bool> = Vec::with_capacity(num_rings);
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

        let rz_scaled = p.radius_z * ratio_z * p.part_scale[2];
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
            scale_ring(&mut coords, p.part_scale);
        }

        let is_tip = ratio_x < 1e-6 && ratio_z < 1e-6;

        // 内环从 clamp+scale 后、剪切前的坐标收缩（与圆柱不同，忠实保留旧语义）
        let inner_coords: Option<Vec<Vec3>> = hollow.then(|| {
            coords
                .iter()
                .map(|&[x, yy, z]| {
                    let s = shrink_radial(x, z, yy, wall);
                    [s[0], shear_y(s[1], s[2], rz_scaled, cut_depth), s[2]]
                })
                .collect()
        });
        let sheared: Vec<Vec3> = coords
            .iter()
            .map(|&c| [c[0], shear_y(c[1], c[2], rz_scaled, cut_depth), c[2]])
            .collect();

        let ring_idx = emit_ring(&mut mesh, &sheared, t, is_tip.then_some([0.5, t]));
        let inner_idx = inner_coords.map(|ic| {
            if is_tip {
                vec![ring_idx[0]; segments]
            } else {
                emit_ring(&mut mesh, &ic, t, None)
            }
        });
        rings.push(ring_idx);
        tip_flags.push(is_tip);
        if let (Some(ir), Some(ii)) = (&mut inner_rings, inner_idx) {
            ir.push(ii);
        }
    }

    // 环间侧面（尖端环按点退化处理，直接发射三角扇）
    let mut f = Vec::new();
    for ring in 0..subdivisions {
        stitch_side(
            &mut f,
            &rings[ring],
            &rings[ring + 1],
            tip_flags[ring],
            tip_flags[ring + 1],
            false,
        );
    }
    mesh.add_faces(f, FaceGroup::Side);

    let has_bottom_cap = p.bottom_scale[0] > 1e-6 || p.bottom_scale[1] > 1e-6;
    if let Some(ir) = &inner_rings {
        let mut f = Vec::new();
        for ring in 0..subdivisions {
            stitch_side(
                &mut f,
                &ir[ring],
                &ir[ring + 1],
                tip_flags[ring],
                tip_flags[ring + 1],
                true,
            );
        }
        mesh.add_faces(f, FaceGroup::Side);
        if has_bottom_cap {
            let mut f = Vec::new();
            cap_annulus(&mut f, &rings[0], &ir[0], false);
            mesh.add_faces(f, FaceGroup::Cap);
        }
    } else if has_bottom_cap {
        let [sx, sy, sz] = p.part_scale;
        let center = [-p.offset[0] * sx, -half_len * sy, -p.offset[2] * sz];
        let c = mesh.push_vertex(center, [0.5, 0.5]);
        let mut f = Vec::new();
        cap_fan(&mut f, c, &rings[0], false);
        mesh.add_faces(f, FaceGroup::Cap);
    }

    // 鼻锥为光滑旋转曲面：环向 perimeter 边强制平滑（消除分段轮廓折痕），
    // 竖直边仍走角度判定（teardrop 捏折等保留硬边）
    for ring in &rings {
        mesh.smooth_edges.extend(perimeter_edges(ring));
    }
    if let Some(ir) = &inner_rings {
        for ring in ir {
            mesh.smooth_edges.extend(perimeter_edges(ring));
        }
    }

    // 端环：鼻锥底环供接头法线混合（尖端为自由点，不参与）
    if !rings.is_empty() && !tip_flags[0] {
        mesh.rims.push(Rim {
            vertices: rings[0].clone(),
            smooth: p.flatten_normals.smooth_bottom(),
        });
    }

    Some(mesh)
}
