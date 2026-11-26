use super::*;
use crate::core::parse::model::{FlattenNormals, FuselageParams};
use crate::core::parse::{Part, PartType};

fn cone(wall: f64) -> Part {
    Part {
        id: "t".into(),
        part_type: PartType::NoseCone,
        position: [0.0; 3],
        rotation: [0.0; 3],
        material_index: 0,
        material_name: "m".into(),
        fuselage: Some(FuselageParams {
            length: 2.0,
            radius_x: 1.0,
            radius_z: 1.0,
            offset: [0.0; 3],
            top_scale: [0.0; 2],
            bottom_scale: [1.0; 2],
            deformations: [0.0; 3],
            corner_radiuses: [1.0; 8],
            clamp_distances: [1.0; 8],
            part_scale: [1.0; 3],
            flatten_normals: FlattenNormals::None,
        }),
        wall_thickness: wall,
        subdivisions: 5,
    }
}

/// 多环鼻锥：尖端单顶点入 apex、环向边强制平滑、底环供接头
#[test]
fn multi_ring_tip_apex_and_smooth_perimeter() {
    let m = generate(&cone(0.0), 8).unwrap();
    assert_eq!(m.apex_vertices.len(), 1);
    assert!(!m.smooth_edges.is_empty());
    assert_eq!(m.rims.len(), 1);
    let tip = *m.apex_vertices.iter().next().unwrap();
    assert!((m.vertices[tip][1] - 1.0).abs() < 1e-9);
}

/// 空心鼻锥：内外尖点独立（apex=2），内尖点内缩
#[test]
fn hollow_cone_separate_apex() {
    let m = generate(&cone(0.1), 8).unwrap();
    assert_eq!(m.apex_vertices.len(), 2);
    let mut ys: Vec<f64> = m.apex_vertices.iter().map(|&v| m.vertices[v][1]).collect();
    ys.sort_by(f64::total_cmp);
    assert!(ys[0] < ys[1] - 1e-6);
}
