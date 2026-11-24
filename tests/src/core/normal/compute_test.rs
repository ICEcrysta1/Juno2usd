use super::*;
use crate::core::parse::model::{FlattenNormals, FuselageParams};
use crate::core::parse::{Part, PartType};
use crate::core::shapes;
use crate::core::shapes::algo::transform::{cross, dot, normalize};

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
            flatten_normals: FlattenNormals::None,
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

/// 鼻锥尖端：入射边强制锐边 → 尖端面角法线 = 面法线（尖锐，不圆滑）
#[test]
fn nose_cone_apex_sharp() {
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
        if !f.contains(&tip) {
            continue;
        }
        let p1 = mesh.vertices[f[0]];
        let p2 = mesh.vertices[f[1]];
        let p3 = mesh.vertices[f[2]];
        let fnrm = normalize(cross(
            [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]],
            [p3[0] - p1[0], p3[1] - p1[1], p3[2] - p1[2]],
        ));
        for k in 0..3 {
            if f[k] == tip {
                let n = ns[fi * 3 + k];
                assert!(dot(n, fnrm) > 0.99, "尖端应锐边（=面法线），实际 {n:?}");
                checked += 1;
            }
        }
    }
    assert!(checked > 0, "未找到尖端面");
}

/// 缩 0 端（非圆锥）：顶点硬角（=面法线），但母线在底环处仍平滑（≠面法线）
#[test]
fn fuselage_point_end_hard_tip_smooth_side() {
    let mut part = cylinder_part([1.0; 8]);
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
    let face_n = |f: &[usize; 3]| {
        let p1 = mesh.vertices[f[0]];
        let p2 = mesh.vertices[f[1]];
        let p3 = mesh.vertices[f[2]];
        normalize(cross(
            [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]],
            [p3[0] - p1[0], p3[1] - p1[1], p3[2] - p1[2]],
        ))
    };
    // 顶点硬角：面角法线 = 面法线
    let mut checked_tip = 0;
    for (fi, f) in mesh.faces.iter().enumerate() {
        if !f.contains(&tip) {
            continue;
        }
        let fnrm = face_n(f);
        for k in 0..3 {
            if f[k] == tip {
                assert!(dot(ns[fi * 3 + k], fnrm) > 0.99, "缩0端顶点应硬角");
                checked_tip += 1;
            }
        }
    }
    assert!(checked_tip > 0, "未找到尖端面");
    // 底环侧面平滑：母线未标锐 → 底环面角法线被平均（≠面法线）
    let base_y = mesh.vertices[tip][1] - 2.0;
    let mut smoothed = 0;
    for (fi, f) in mesh.faces.iter().enumerate() {
        if mesh.face_groups[fi] != FaceGroup::Side {
            continue;
        }
        let fnrm = face_n(f);
        for k in 0..3 {
            if (mesh.vertices[f[k]][1] - base_y).abs() < 1e-6 && dot(ns[fi * 3 + k], fnrm) < 0.999 {
                smoothed += 1;
            }
        }
    }
    assert!(smoothed > 0, "底环母线应平滑（法线被平均），不应整面硬棱");
}

/// 空心缩 0 端：内外尖点各自独立顶点（不合并）
#[test]
fn hollow_point_end_separate_apex() {
    let mut part = cylinder_part([1.0; 8]);
    part.wall_thickness = 0.1;
    if let Some(p) = &mut part.fuselage {
        p.top_scale = [0.0; 2];
    }
    let mesh = shapes::generate(&part, 24).unwrap();
    assert_eq!(mesh.hard_vertices.len(), 2, "空心缩0端应有内外两个独立硬点");
    let mut ys: Vec<f64> = mesh
        .hard_vertices
        .iter()
        .map(|&v| mesh.vertices[v][1])
        .collect();
    ys.sort_by(|a, b| a.total_cmp(b));
    let tip_y = mesh.vertices.iter().map(|v| v[1]).fold(f64::MIN, f64::max);
    assert!((ys[1] - tip_y).abs() < 1e-6, "外尖点应在顶端");
    assert!(ys[0] < ys[1] - 1e-6, "内尖点应内缩，保留壁厚");
}

/// 空心鼻锥尖：内外尖点各自独立顶点（不合并）
#[test]
fn hollow_nose_cone_separate_apex() {
    let mut part = cylinder_part([1.0; 8]);
    part.part_type = PartType::NoseCone;
    part.wall_thickness = 0.1;
    if let Some(p) = &mut part.fuselage {
        p.top_scale = [0.0; 2];
    }
    let mesh = shapes::generate(&part, 24).unwrap();
    assert_eq!(mesh.apex_vertices.len(), 2, "空心鼻锥应有内外两个独立尖点");
    let mut ys: Vec<f64> = mesh
        .apex_vertices
        .iter()
        .map(|&v| mesh.vertices[v][1])
        .collect();
    ys.sort_by(|a, b| a.total_cmp(b));
    assert!(ys[0] < ys[1] - 1e-6, "内尖点应内缩，保留壁厚");
}
