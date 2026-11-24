use super::*;
use crate::core::parse::connection::ConnectionTree;
use crate::core::parse::model::{FlattenNormals, FuselageParams};
use crate::core::parse::{Part, PartType};
use crate::core::shapes;

/// 重导出的法线编排/接头混合入口可用
#[test]
fn reexport_normal_entries_run() {
    let part = Part {
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
            corner_radiuses: [1.0; 8],
            clamp_distances: [1.0; 8],
            part_scale: [1.0; 3],
            flatten_normals: FlattenNormals::None,
        }),
        wall_thickness: 0.0,
        subdivisions: 5,
    };
    let mesh = shapes::generate(&part, 8).unwrap();
    let ns = compute(&mesh, 60.0);
    assert_eq!(ns.len(), mesh.faces.len() * 3);
    blend_joints(
        &[mesh],
        &mut [ns],
        &["t".into()],
        &ConnectionTree::default(),
        60.0,
    );
}
