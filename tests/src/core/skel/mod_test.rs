use super::*;
use crate::core::parse::connection::ConnectionTree;
use crate::core::parse::{Part, PartType};

/// 重导出的骨骼构建入口可用
#[test]
fn reexport_build_runs() {
    let part = Part {
        id: "j".into(),
        part_type: PartType::Rotator,
        position: [0.0; 3],
        rotation: [0.0; 3],
        material_index: 0,
        material_name: "m".into(),
        fuselage: None,
        wall_thickness: 0.0,
        subdivisions: 5,
    };
    let sk = build(&[part], &ConnectionTree::default());
    assert_eq!(sk.joints.len(), 1);
    assert_eq!(sk.joints[0].path, "joint_j");
}
