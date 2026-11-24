use super::*;
use crate::core::parse::model::{FlattenNormals, FuselageParams};
use crate::core::parse::{Part, PartType};

fn part() -> Part {
    Part {
        id: "t".into(),
        part_type: PartType::Fuselage,
        position: [1.0, 2.0, 3.0],
        rotation: [0.0, 90.0, 0.0],
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
            corner_radiuses: [1.0; 8],
            clamp_distances: [1.0; 8],
            part_scale: [1.0; 3],
            flatten_normals: FlattenNormals::None,
        }),
        wall_thickness: 0.0,
        subdivisions: 5,
    }
}

/// 顶点/UV 一一对应，索引递增
#[test]
fn push_vertex_aligns_uv() {
    let mut m = RawMesh::new(&part());
    assert_eq!(m.push_vertex([0.0; 3], [0.1, 0.2]), 0);
    assert_eq!(m.push_vertex([1.0; 3], [0.3, 0.4]), 1);
    assert_eq!(m.uvs.len(), 2);
    assert_eq!(m.uvs[1], [0.3, 0.4]);
}

/// 面/四边面批量追加并打标签
#[test]
fn add_faces_and_quads_tag_groups() {
    let mut m = RawMesh::new(&part());
    m.add_faces(vec![[0, 1, 2]], FaceGroup::Cap);
    m.add_quads(vec![[0, 1, 2, 3]], FaceGroup::Side);
    assert_eq!(m.face_groups, vec![FaceGroup::Cap]);
    assert_eq!(m.quad_groups, vec![FaceGroup::Side]);
}

/// 局部→世界：R·v + position（Y90°: x→-z）
#[test]
fn to_world_rotation_and_translation() {
    let p = part();
    let mut m = RawMesh::new(&p);
    m.push_vertex([1.0, 0.0, 0.0], [0.0, 0.0]);
    let w = m.to_world(&p);
    let v = w.vertices[0];
    assert!((v[0] - 1.0).abs() < 1e-9 && (v[1] - 2.0).abs() < 1e-9 && (v[2] - 2.0).abs() < 1e-9);
}
