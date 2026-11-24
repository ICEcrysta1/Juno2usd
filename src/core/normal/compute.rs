//! 法线编排：侧面跑 Blender 自动平滑，端盖单独算平面法线，输出 faceVarying 法线。
//!
//! 侧面/端盖分 pass：焊接对接环边被侧+盖共 4 面共享，统一 pass 会判非流形锐边
//! 而拆开侧面、破坏零件间平滑；分 pass 后侧面（焊接）光滑、端盖硬边。
//! 世界变换由调用方经 algo::transform 完成（或直接在合并后的世界网格上计算）。

use super::smooth;
use crate::core::shapes::RawMesh;
use crate::core::shapes::algo::transform::{Vec3, cross, dot, normalize};
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
    let ns = smooth::smooth_normals(
        &mesh.vertices,
        &side_faces,
        sharp_edge_angle,
        &mesh.smooth_edges,
    );
    for (k, &fi) in side.iter().enumerate() {
        out[fi * 3..fi * 3 + 3].copy_from_slice(&ns[k * 3..k * 3 + 3]);
    }

    // 端盖：平面法线（朝向由绕序决定），整面共享→硬边
    for &fi in &cap {
        let f = mesh.faces[fi];
        let p1 = mesh.vertices[f[0]];
        let p2 = mesh.vertices[f[1]];
        let p3 = mesh.vertices[f[2]];
        let n = normalize(cross(
            [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]],
            [p3[0] - p1[0], p3[1] - p1[1], p3[2] - p1[2]],
        ));
        let n = if dot(n, n) > 0.5 { n } else { [0.0, 1.0, 0.0] };
        for k in 0..3 {
            out[fi * 3 + k] = n;
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::parse::model::FuselageParams;
    use crate::core::parse::{Part, PartType};
    use crate::core::shapes;
    use crate::core::shapes::algo::transform::{dot, normalize};

    fn cylinder_part(corners: [f64; 8]) -> Part {
        Part {
            id: "t".into(),
            part_type: PartType::Fuselage,
            position: [0.0; 3],
            rotation: [0.0; 3],
            material_index: 0,
            material_name: "m".into(),
            fuselage: Some(FuselageParams {
                length: 2.0,
                radius_x: 1.0,
                radius_z: 1.0,
                offset: [0.0; 3],
                top_scale: [1.0; 2],
                bottom_scale: [1.0; 2],
                deformations: [0.0; 3],
                corner_radiuses: corners,
                clamp_distances: [1.0; 8],
                part_scale: [1.0; 3],
            }),
            wall_thickness: 0.0,
            subdivisions: 5,
        }
    }

    /// 圆柱：侧面法线径向；垂直端盖（90°>60°）自动硬边→轴向
    #[test]
    fn cylinder_side_radial_caps_axial() {
        let part = cylinder_part([1.0; 8]);
        let mesh = shapes::generate(&part, 24).unwrap();
        let ns = compute(&mesh, 60.0);
        let mut saw_cap = false;
        for (fi, f) in mesh.faces.iter().enumerate() {
            for k in 0..3 {
                let n = ns[fi * 3 + k];
                let v = mesh.vertices[f[k]];
                if n[1].abs() > 0.99 {
                    // 端盖法线轴向
                    saw_cap = true;
                } else {
                    // 侧面法线径向
                    let radial = normalize([v[0], 0.0, v[2]]);
                    assert!(dot(n, radial) > 0.99, "侧面法线非径向: {n:?}");
                }
            }
        }
        assert!(saw_cap, "应存在轴向端盖法线");
    }

    /// 方形截面：90° 棱 > 60° → 硬边，侧面法线轴对齐
    #[test]
    fn square_cross_section_hard_edges() {
        let part = cylinder_part([0.0; 8]);
        let mesh = shapes::generate(&part, 24).unwrap();
        let ns = compute(&mesh, 60.0);
        for (fi, _) in mesh.faces.iter().enumerate() {
            for k in 0..3 {
                let n = ns[fi * 3 + k];
                if n[1].abs() > 0.99 {
                    continue; // 端盖
                }
                let axis = n[0].abs().max(n[2].abs());
                assert!(axis > 0.99, "方形侧面法线非轴对齐: {n:?}");
            }
        }
    }

    /// 鼻锥尖端：60° 下扇面平滑 → 尖端法线轴向（亮尖点）
    #[test]
    fn nose_cone_apex_axial() {
        let mut part = cylinder_part([1.0; 8]);
        part.part_type = PartType::NoseCone;
        if let Some(p) = &mut part.fuselage {
            p.top_scale = [0.0; 2];
        }
        let mesh = shapes::generate(&part, 24).unwrap();
        let ns = compute(&mesh, 60.0);
        let tip = mesh
            .vertices
            .iter()
            .enumerate()
            .max_by(|a, b| a.1[1].total_cmp(&b.1[1]))
            .unwrap()
            .0;
        let mut checked = 0;
        for (fi, f) in mesh.faces.iter().enumerate() {
            for k in 0..3 {
                if f[k] == tip {
                    let n = ns[fi * 3 + k];
                    assert!(dot(n, [0.0, 1.0, 0.0]) > 0.99, "尖端法线非轴向: {n:?}");
                    checked += 1;
                }
            }
        }
        assert!(checked > 0, "未找到尖端面");
    }
}
