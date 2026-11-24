use super::*;
use crate::core::skel::build::Joint;

fn one_joint() -> Skeleton {
    let id = [
        1.0, 0.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        0.0, 0.0, 0.0, 1.0,
    ];
    Skeleton {
        joints: vec![Joint {
            path: "joint_j".into(),
            part_id: "j".into(),
            bind: id,
            rest: id,
        }],
        part_joint: [("j".into(), 0)].into_iter().collect(),
    }
}

/// SkelRoot/Skeleton 三属性；网格 100% 单关节绑定 primvar
#[test]
fn define_skeleton_and_bind_mesh() {
    let (stage, root) = super::super::stage::create("T").unwrap();
    let sp = define(&stage, &format!("/{root}"), &one_joint()).unwrap();
    assert_eq!(sp, format!("/{root}/SkelRoot/Skeleton"));
    stage
        .define_typed_prim(format!("/{root}/M"), "Mesh")
        .unwrap();
    bind(&stage, &format!("/{root}/M"), &sp, 0, 3).unwrap();
    let text = stage.root_layer().borrow().export_to_string().unwrap();
    assert!(text.contains("SkelRoot"));
    assert!(text.contains("bindTransforms"));
    assert!(text.contains("restTransforms"));
    assert!(text.contains("joint_j"));
    assert!(text.contains("primvars:skel:jointIndices = [0, 0, 0]"));
    assert!(text.contains("primvars:skel:jointWeights = [1, 1, 1]"));
}
