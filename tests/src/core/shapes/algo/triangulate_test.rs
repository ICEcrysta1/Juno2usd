use super::*;
use crate::core::shapes::components::model::FaceGroup;

fn mesh_with(verts: Vec<Vec3>, quads: Vec<[usize; 4]>) -> RawMesh {
    let n = verts.len();
    let qn = quads.len();
    RawMesh {
        part_id: "t".into(),
        material_name: "m".into(),
        vertices: verts,
        uvs: vec![[0.0, 0.0]; n],
        faces: Vec::new(),
        face_groups: Vec::new(),
        quads,
        quad_groups: vec![FaceGroup::Side; qn],
        smooth_edges: Default::default(),
        sharp_edges: Default::default(),
        apex_vertices: Default::default(),
        hard_vertices: Default::default(),
        rims: Vec::new(),
    }
}

/// 塌缩四边形 → 单三角；完整四边面 → 两三角，标签随面
#[test]
fn collapsed_and_full_quad() {
    let mut m = mesh_with(
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
        vec![[0, 0, 1, 2]],
    );
    triangulate(&mut m);
    assert_eq!(m.faces, vec![[0, 1, 2]]);
    assert!(m.quads.is_empty());

    let mut sq = mesh_with(
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ],
        vec![[0, 1, 2, 3]],
    );
    triangulate(&mut sq);
    // 平面正方形两对角线等长 → 平局取第一条
    assert_eq!(sq.faces, vec![[0, 1, 2], [0, 2, 3]]);
    assert_eq!(sq.face_groups, vec![FaceGroup::Side; 2]);
}

/// 非平面四边形：所选对角线两三角法线对 quad 法线偏差最小
#[test]
fn non_planar_picks_min_deviation_diagonal() {
    let verts = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 1.0],
        [0.0, 1.0, 0.0],
    ];
    let mut m = mesh_with(verts.clone(), vec![[0, 1, 2, 3]]);
    triangulate(&mut m);
    let qn = ring_normal(&verts, &[0, 1, 2, 3]);
    let dev = |t1: [usize; 3], t2: [usize; 3]| {
        (1.0 - dot(face_normal(&verts, &t1), qn)) + (1.0 - dot(face_normal(&verts, &t2), qn))
    };
    let chosen = dev(m.faces[0], m.faces[1]);
    let other = if m.faces[0] == [0, 1, 2] {
        dev([0, 1, 3], [1, 2, 3])
    } else {
        dev([0, 1, 2], [0, 2, 3])
    };
    assert!(chosen <= other + 1e-9);
}
