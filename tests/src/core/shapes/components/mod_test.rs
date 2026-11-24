use super::*;
use crate::core::parse::model::{FlattenNormals, FuselageParams};
use crate::core::parse::{Part, PartType};

fn part(t: PartType) -> Part {
    Part {
        id: "t".into(),
        part_type: t,
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
            corner_radiuses: [1.0; 8],
            clamp_distances: [1.0; 8],
            part_scale: [1.0; 3],
            flatten_normals: FlattenNormals::None,
        }),
        wall_thickness: 0.0,
        subdivisions: 5,
    }
}

/// 类型分发：几何件 Some、关节件 None
#[test]
fn dispatch_by_part_type() {
    assert!(generate(&part(PartType::Fuselage), 8).is_some());
    assert!(generate(&part(PartType::Strut), 8).is_some());
    assert!(generate(&part(PartType::NoseCone), 8).is_some());
    assert!(generate(&part(PartType::Inlet), 8).is_some());
    assert!(generate(&part(PartType::Fairing), 8).is_some());
    assert!(generate(&part(PartType::FairingBase), 8).is_some());
    assert!(generate(&part(PartType::FairingNoseCone), 8).is_some());
    assert!(generate(&part(PartType::Rotator), 8).is_none());
    assert!(generate(&part(PartType::HingeRotator), 8).is_none());
}
