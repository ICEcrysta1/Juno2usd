use super::*;
use crate::core::parse::connection::Connection;
use crate::core::parse::model::FuselageParams;
use crate::core::parse::{Part, PartType};

fn part(id: &str, t: PartType, pos: Vec3, rot: Vec3) -> Part {
    Part {
        id: id.into(),
        part_type: t,
        position: pos,
        rotation: rot,
        material_index: 0,
        material_name: "m".into(),
        fuselage: (t == PartType::Fuselage).then(|| FuselageParams {
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
            flatten_normals: Default::default(),
        }),
        wall_thickness: 0.0,
        subdivisions: 5,
    }
}

/// 关节层级：沿零件树找关节父；rest 行向量约定；BFS 绑定遇关节停止
#[test]
fn joint_hierarchy_rest_and_bindings() {
    let parts = vec![
        part("j1", PartType::Rotator, [0.0; 3], [0.0; 3]),
        part("p1", PartType::Fuselage, [0.0, 1.0, 0.0], [0.0; 3]),
        part("j2", PartType::Rotator, [0.0, 2.0, 0.0], [0.0, 90.0, 0.0]),
        part("p2", PartType::Fuselage, [5.0, 0.0, 0.0], [0.0; 3]),
    ];
    let tree = ConnectionTree {
        connections: vec![
            Connection {
                part_a: "p1".into(),
                part_b: "j1".into(),
                attach_a: String::new(),
                attach_b: String::new(),
            },
            Connection {
                part_a: "j2".into(),
                part_b: "p1".into(),
                attach_a: String::new(),
                attach_b: String::new(),
            },
        ],
        child_to_parent: [("p1".into(), "j1".into()), ("j2".into(), "p1".into())]
            .into_iter()
            .collect(),
        ..Default::default()
    };
    let sk = build(&parts, &tree);
    assert_eq!(sk.joints.len(), 2);
    assert_eq!(sk.joints[0].path, "joint_j1");
    assert_eq!(sk.joints[1].path, "joint_j1/joint_j2");
    // j2 相对 j1：旋转块 Rᵀ（Y90°），平移 Rᵀ·(t2-t1)=[0,2,0]
    let rest = sk.joints[1].rest;
    assert!((rest[2] + 1.0).abs() < 1e-9 && (rest[8] - 1.0).abs() < 1e-9);
    assert_eq!(&rest[12..15], &[0.0, 2.0, 0.0]);
    assert_eq!(&sk.joints[1].bind[12..15], &[0.0, 2.0, 0.0]);
    // 绑定：关节绑自己；p1 经 BFS 绑 j1；孤立 p2 缺席
    assert_eq!(sk.part_joint["j1"], 0);
    assert_eq!(sk.part_joint["j2"], 1);
    assert_eq!(sk.part_joint["p1"], 0);
    assert!(!sk.part_joint.contains_key("p2"));
}

/// 无关节 → 空骨骼
#[test]
fn no_joints_empty_skeleton() {
    let parts = vec![part("p", PartType::Fuselage, [0.0; 3], [0.0; 3])];
    let sk = build(&parts, &ConnectionTree::default());
    assert!(sk.joints.is_empty() && sk.part_joint.is_empty());
}
