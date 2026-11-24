use super::super::compute;
use super::*;
use crate::core::parse::connection::Connection;
use crate::core::parse::model::{FlattenNormals, FuselageParams};
use crate::core::parse::{Part, PartType};
use crate::core::shapes;

fn part(id: &str, pos: Vec3, rot: Vec3, flat: FlattenNormals) -> Part {
    Part {
        id: id.into(),
        part_type: PartType::Fuselage,
        position: pos,
        rotation: rot,
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
            flatten_normals: flat,
        }),
        wall_thickness: 0.0,
        subdivisions: 5,
    }
}

/// 堆叠 coaxial 圆柱（B 旋转半格使对接环法线方向不同）
fn stacked(flat_a: FlattenNormals, flat_b: FlattenNormals) -> (Vec<RawMesh>, Vec<Vec<Vec3>>) {
    let a = part("a", [0.0, 1.0, 0.0], [0.0; 3], flat_a);
    // 旋转 20°（非半格）保证最近邻配对唯一，避免等距碰撞
    let b = part("b", [0.0, 3.0, 0.0], [0.0, 20.0, 0.0], flat_b);
    let meshes: Vec<RawMesh> = [&a, &b]
        .iter()
        .map(|p| shapes::generate(p, 8).unwrap().to_world(p))
        .collect();
    let normals: Vec<Vec<Vec3>> = meshes.iter().map(|m| compute(m, 60.0)).collect();
    (meshes, normals)
}

fn tree() -> ConnectionTree {
    ConnectionTree {
        connections: vec![Connection {
            part_a: "a".into(),
            part_b: "b".into(),
            attach_a: String::new(),
            attach_b: String::new(),
        }],
        ..Default::default()
    }
}

fn radial(v: Vec3) -> Vec3 {
    normalize([v[0], 0.0, v[2]])
}

/// 双开：对接环互相平均 → 配对顶点法线一致且偏离纯径向
#[test]
fn both_smooth_averages_rim_normals() {
    let (meshes, mut normals) = stacked(FlattenNormals::Top, FlattenNormals::Bottom);
    blend_joints(
        &meshes,
        &mut normals,
        &["a".into(), "b".into()],
        &tree(),
        60.0,
    );
    let ra = &meshes[0].rims[1];
    let rb = &meshes[1].rims[0];
    for &va in &ra.vertices {
        let kb = nearest(&meshes[0], va, &meshes[1], &rb.vertices);
        let na = vertex_side_normal(&meshes[0], &normals[0], va);
        let nb = vertex_side_normal(&meshes[1], &normals[1], rb.vertices[kb]);
        assert!(dot(na, nb) > 0.999, "双开应互相平均");
        assert!(
            dot(na, radial(meshes[0].vertices[va])) < 0.99,
            "应偏离纯径向"
        );
    }
}

/// 双关：保持径向不平滑
#[test]
fn both_closed_keeps_radial() {
    let (meshes, mut normals) = stacked(FlattenNormals::None, FlattenNormals::None);
    blend_joints(
        &meshes,
        &mut normals,
        &["a".into(), "b".into()],
        &tree(),
        60.0,
    );
    for &va in &meshes[0].rims[1].vertices {
        let na = vertex_side_normal(&meshes[0], &normals[0], va);
        assert!(dot(na, radial(meshes[0].vertices[va])) > 0.99);
    }
}

/// 单开：开侧对齐关侧（关侧保持径向）
#[test]
fn single_smooth_aligns_to_closed() {
    let (meshes, mut normals) = stacked(FlattenNormals::Top, FlattenNormals::None);
    blend_joints(
        &meshes,
        &mut normals,
        &["a".into(), "b".into()],
        &tree(),
        60.0,
    );
    let rb = &meshes[1].rims[0];
    for &va in &meshes[0].rims[1].vertices {
        let kb = nearest(&meshes[0], va, &meshes[1], &rb.vertices);
        let na = vertex_side_normal(&meshes[0], &normals[0], va);
        assert!(
            dot(na, radial(meshes[1].vertices[rb.vertices[kb]])) > 0.99,
            "开侧应对齐关侧径向"
        );
    }
    for &vb in &meshes[1].rims[0].vertices {
        let nb = vertex_side_normal(&meshes[1], &normals[1], vb);
        assert!(dot(nb, radial(meshes[1].vertices[vb])) > 0.99);
    }
}
