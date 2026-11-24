use super::*;
use crate::core::parse::model::{FlattenNormals, FuselageParams};
use crate::core::parse::{Part, PartType};

/// Inlet 复用共享圆柱：与 fuselage 同参输出一致（空心内环）
#[test]
fn inlet_reuses_fuselage_cylinder() {
    let part = Part {
        id: "t".into(),
        part_type: PartType::Inlet,
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
        wall_thickness: 0.1,
        subdivisions: 5,
    };
    let a = generate(&part, 8).unwrap();
    let b = super::super::fuselage::generate(&part, 8).unwrap();
    assert_eq!(a.vertices, b.vertices);
    assert_eq!(a.faces, b.faces);
    assert_eq!(a.vertices.len(), 32);
}
