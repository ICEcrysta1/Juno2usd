use super::*;
use crate::core::shapes::RawMesh;
use crate::core::shapes::components::model::FaceGroup;

/// 重导出的导出入口可用
#[test]
fn reexport_export_runs() {
    let dir = std::env::temp_dir().join(format!("juno2usd-usdamod-{}", std::process::id()));
    let out = dir.join("m.usda");
    let mesh = RawMesh {
        part_id: "p".into(),
        material_name: String::new(),
        vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        uvs: vec![[0.0, 0.0]; 3],
        faces: vec![[0, 1, 2]],
        face_groups: vec![FaceGroup::Side],
        quads: Vec::new(),
        quad_groups: Vec::new(),
        smooth_edges: Default::default(),
        sharp_edges: Default::default(),
        apex_vertices: Default::default(),
        hard_vertices: Default::default(),
        rims: Vec::new(),
    };
    let normals = vec![vec![[0.0, 0.0, 1.0]; 3]];
    export(&out, "T", &[], &[mesh], &normals, None).unwrap();
    let text = std::fs::read_to_string(&out).unwrap();
    assert!(text.contains("def Mesh"));
    std::fs::remove_dir_all(&dir).unwrap();
}
