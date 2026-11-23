//! 法线编排：按面分组分发，输出与面一一对应的 faceVarying 法线（局部空间）。

use super::{cap, smooth};
use crate::core::shapes::algo::transform::Vec3;
use crate::core::shapes::components::model::{FaceGroup, RawMesh};
use std::collections::HashMap;

/// 输出长度 = 3 × 面数，每面三个顶点法线按序排列
pub fn compute(mesh: &RawMesh, sharp_edge_angle: f64) -> Vec<Vec3> {
    let mut out = vec![[0.0; 3]; mesh.faces.len() * 3];

    let mut by_group: HashMap<FaceGroup, Vec<usize>> = HashMap::new();
    for (fi, g) in mesh.face_groups.iter().enumerate() {
        by_group.entry(*g).or_default().push(fi);
    }

    for (g, idxs) in by_group {
        let faces: Vec<[usize; 3]> = idxs.iter().map(|&i| mesh.faces[i]).collect();
        let normals = if g.is_side() {
            smooth::smooth_normals(&mesh.vertices, &faces, sharp_edge_angle, &mesh.sharp_edges)
        } else {
            // 端盖为平面：整组共享一个平面法线
            vec![cap::cap_normal(&mesh.vertices, &faces); faces.len() * 3]
        };
        for (k, &fi) in idxs.iter().enumerate() {
            out[fi * 3..fi * 3 + 3].copy_from_slice(&normals[k * 3..k * 3 + 3]);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::parse::model::{FlattenNormals, FuselageParams};
    use crate::core::parse::{Part, PartType};
    use crate::core::shapes;
    use crate::core::shapes::algo::transform::{dot, normalize};

    fn cylinder_part(corners: [f64; 8], flatten: FlattenNormals) -> Part {
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
                flatten_normals: flatten,
            }),
            wall_thickness: 0.0,
            subdivisions: 5,
        }
    }

    /// 扁平端（Both）：侧面法线径向、端盖法线轴向硬边
    #[test]
    fn cylinder_side_radial_caps_axial() {
        let part = cylinder_part([1.0; 8], FlattenNormals::Both);
        let mesh = shapes::generate(&part, 24).unwrap();
        let ns = compute(&mesh, 40.0);
        for (fi, f) in mesh.faces.iter().enumerate() {
            for k in 0..3 {
                let n = ns[fi * 3 + k];
                let v = mesh.vertices[f[k]];
                match mesh.face_groups[fi] {
                    FaceGroup::OuterSide => {
                        let radial = normalize([v[0], 0.0, v[2]]);
                        assert!(dot(n, radial) > 0.99, "侧面法线非径向: {n:?}");
                        assert!(n[1].abs() < 1e-6, "侧面法线含 Y 分量: {n:?}");
                    }
                    FaceGroup::CapBottom => {
                        assert!(dot(n, [0.0, -1.0, 0.0]) > 0.999, "底盖法线错误: {n:?}")
                    }
                    FaceGroup::CapTop => {
                        assert!(dot(n, [0.0, 1.0, 0.0]) > 0.999, "顶盖法线错误: {n:?}")
                    }
                    FaceGroup::InnerSide => {}
                }
            }
        }
    }

    /// 平滑端（None）：端盖并入侧面组，边缘法线向轴向倾斜（零件间光滑过渡）
    #[test]
    fn smooth_end_rim_normals_blend() {
        let part = cylinder_part([1.0; 8], FlattenNormals::None);
        let mesh = shapes::generate(&part, 24).unwrap();
        let ns = compute(&mesh, 40.0);
        let mut rim_tilted = false;
        for (fi, f) in mesh.faces.iter().enumerate() {
            if mesh.face_groups[fi] != FaceGroup::OuterSide {
                continue;
            }
            for k in 0..3 {
                let n = ns[fi * 3 + k];
                let v = mesh.vertices[f[k]];
                // 顶环边缘顶点（y=+1）：法线应含 +Y 分量（与端盖混合）
                if (v[1] - 1.0).abs() < 1e-9 && n[1] > 0.1 {
                    rim_tilted = true;
                }
            }
        }
        assert!(rim_tilted, "平滑端边缘法线未向轴向倾斜");
    }

    /// 方形截面：锐边拆分后侧面法线轴对齐（硬边存在）
    #[test]
    fn square_cross_section_hard_edges() {
        let part = cylinder_part([0.0; 8], FlattenNormals::Both);
        let mesh = shapes::generate(&part, 24).unwrap();
        let ns = compute(&mesh, 40.0);
        for (fi, _) in mesh.faces.iter().enumerate() {
            if mesh.face_groups[fi] != FaceGroup::OuterSide {
                continue;
            }
            for k in 0..3 {
                let f = mesh.faces[fi];
                let n = ns[fi * 3 + k];
                let v = mesh.vertices[f[k]];
                // 端盖已扁平分离，侧面顶点法线轴对齐
                let axis = n[0].abs().max(n[2].abs());
                assert!(axis > 0.99, "方形侧面法线非轴对齐: {n:?} (v={v:?})");
            }
        }
    }

    /// 鼻锥尖端：组成尖端的边标锐 → 相邻扇面在尖端法线不同（星芒收尖），
    /// 且尖端法线接近各自扇面法线（不被环带稀释）
    #[test]
    fn nose_cone_apex_sharp_split() {
        let mut part = cylinder_part([1.0; 8], FlattenNormals::None);
        part.part_type = PartType::NoseCone;
        if let Some(p) = &mut part.fuselage {
            p.top_scale = [0.0; 2];
        }
        let mesh = shapes::generate(&part, 24).unwrap();
        let ns = compute(&mesh, 40.0);
        let tip = mesh
            .vertices
            .iter()
            .enumerate()
            .max_by(|a, b| a.1[1].total_cmp(&b.1[1]))
            .unwrap()
            .0;
        assert!(!mesh.sharp_edges.is_empty(), "尖端应声明锐边");
        // 收集含尖端的面及其尖端法线
        let mut apex_normals: Vec<Vec3> = Vec::new();
        for (fi, f) in mesh.faces.iter().enumerate() {
            if mesh.face_groups[fi] != FaceGroup::OuterSide {
                continue;
            }
            for k in 0..3 {
                if f[k] == tip {
                    apex_normals.push(ns[fi * 3 + k]);
                }
            }
        }
        assert!(apex_normals.len() >= 24, "尖端扇面数不足");
        // 相邻扇面尖端法线应不同（锐边拆分），且都带 +Y 分量（朝上收尖）
        let distinct = apex_normals.windows(2).any(|w| dot(w[0], w[1]) < 0.999);
        assert!(distinct, "尖端扇面法线未拆分（仍被平均成圆顶）");
        assert!(apex_normals.iter().all(|n| n[1] > 0.0), "尖端法线应朝上");
    }
}
