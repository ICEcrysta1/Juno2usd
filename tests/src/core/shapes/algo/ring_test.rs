use super::*;

fn empty_mesh() -> RawMesh {
    RawMesh {
        part_id: "t".into(),
        material_name: "m".into(),
        vertices: Vec::new(),
        uvs: Vec::new(),
        faces: Vec::new(),
        face_groups: Vec::new(),
        quads: Vec::new(),
        quad_groups: Vec::new(),
        smooth_edges: Default::default(),
        sharp_edges: Default::default(),
        apex_vertices: Default::default(),
        hard_vertices: Default::default(),
        rims: Vec::new(),
    }
}

/// 环发射：逐顶点 UV；point_uv 退化共享单中心顶点
#[test]
fn emit_ring_normal_and_point() {
    let mut m = empty_mesh();
    let coords = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0]];
    let idx = emit_ring(&mut m, &coords, 0.5, None);
    assert_eq!(idx, vec![0, 1, 2]);
    assert_eq!(m.uvs[1], [1.0 / 3.0, 0.5]);
    let pt = emit_ring(&mut m, &coords, 1.0, Some([0.5, 1.0]));
    assert_eq!(pt, vec![3, 3, 3]);
    assert_eq!(m.vertices.len(), 4);
}

/// perimeter 边无序对
#[test]
fn perimeter_edges_unordered() {
    assert_eq!(perimeter_edges(&[2, 0, 1]), vec![(0, 2), (0, 1), (1, 2)]);
}

/// 环间缝合：双环记四边面；点端发三角扇；双点无面
#[test]
fn stitch_side_quads_and_fans() {
    let mut m = empty_mesh();
    let lower = [0, 1, 2];
    let upper = [3, 4, 5];
    stitch_side(&mut m, &lower, &upper, false, false, false);
    assert_eq!(m.quads.len(), 3);
    assert!(m.faces.is_empty());
    assert_eq!(m.quad_groups, vec![FaceGroup::Side; 3]);

    let mut fan = empty_mesh();
    stitch_side(&mut fan, &[0, 0, 0], &upper, true, false, false);
    assert_eq!(fan.faces.len(), 3);
    assert!(fan.quads.is_empty());

    let mut none = empty_mesh();
    stitch_side(&mut none, &[0, 0, 0], &[1, 1, 1], true, true, false);
    assert!(none.faces.is_empty() && none.quads.is_empty());
}

/// 端盖：实心扇 n 三角、空心环 2n 三角；invert 反转绕序
#[test]
fn cap_fan_and_annulus() {
    let mut f = Vec::new();
    cap_fan(&mut f, 9, &[0, 1, 2], false);
    assert_eq!(f, vec![[9, 0, 1], [9, 1, 2], [9, 2, 0]]);
    let mut inv = Vec::new();
    cap_fan(&mut inv, 9, &[0, 1, 2], true);
    assert_eq!(inv[0], [9, 1, 0]);
    let mut a = Vec::new();
    cap_annulus(&mut a, &[0, 1], &[2, 3], false);
    assert_eq!(a.len(), 4);
}
