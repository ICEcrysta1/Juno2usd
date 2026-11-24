use super::*;
use crate::core::parse::model::{FlattenNormals, FuselageParams};
use crate::core::parse::{Part, PartType};

fn cylinder(wall: f64, corners: [f64; 8], flat: FlattenNormals) -> Part {
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
            flatten_normals: flat,
        }),
        wall_thickness: wall,
        subdivisions: 5,
    }
}

/// 实心圆柱：2n+2 顶点、4n 三角（侧面四边面已剖分）、双端环默认关平滑
#[test]
fn solid_cylinder_topology() {
    let m = generate(&cylinder(0.0, [1.0; 8], FlattenNormals::None), 8).unwrap();
    assert_eq!(m.vertices.len(), 18);
    assert_eq!(m.faces.len(), 32);
    assert!(m.quads.is_empty());
    assert_eq!(m.rims.len(), 2);
    assert!(m.rims.iter().all(|r| !r.smooth));
}

/// 方形截面：侧面棱全部强制锐边
#[test]
fn square_section_forces_sharp_edges() {
    let m = generate(&cylinder(0.0, [0.0; 8], FlattenNormals::None), 8).unwrap();
    assert!(!m.sharp_edges.is_empty());
}

/// 缩 0 端：硬点 + 单端环；flattenNormals 覆盖端开平滑
#[test]
fn point_end_hard_vertex_and_flatten_gate() {
    let mut p = cylinder(0.0, [1.0; 8], FlattenNormals::Both);
    p.fuselage.as_mut().unwrap().top_scale = [0.0; 2];
    let m = generate(&p, 8).unwrap();
    assert_eq!(m.hard_vertices.len(), 1);
    assert_eq!(m.rims.len(), 1);
    assert!(m.rims[0].smooth);
}

/// 空心：内外环独立顶点（4n）、环形端盖，无中心顶点
#[test]
fn hollow_inner_rings_and_annulus_caps() {
    let m = generate(&cylinder(0.1, [1.0; 8], FlattenNormals::None), 8).unwrap();
    assert_eq!(m.vertices.len(), 32);
    assert_eq!(m.faces.len(), 64);
}
