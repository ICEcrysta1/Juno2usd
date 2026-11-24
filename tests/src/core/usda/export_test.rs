use super::*;
use crate::core::skel::build::Joint;

fn tri_mesh(id: &str, mat: &str) -> RawMesh {
    RawMesh {
        part_id: id.into(),
        material_name: mat.into(),
        vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        uvs: vec![[0.0, 0.0]; 3],
        faces: vec![[0, 1, 2], [0, 0, 1]], // 第二面退化，应被过滤
        face_groups: vec![crate::core::shapes::components::model::FaceGroup::Side; 2],
        quads: Vec::new(),
        quad_groups: Vec::new(),
        smooth_edges: Default::default(),
        sharp_edges: Default::default(),
        apex_vertices: Default::default(),
        hard_vertices: Default::default(),
        rims: Vec::new(),
    }
}

fn identity() -> [f64; 16] {
    [
        1.0, 0.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        0.0, 0.0, 0.0, 1.0,
    ]
}

/// 无骨骼：同材质合并单 Mesh，退化面过滤
#[test]
fn merge_same_material_and_filter_degenerate() {
    let dir = std::env::temp_dir().join(format!("juno2usd-export-{}", std::process::id()));
    let out = dir.join("merge.usda");
    let meshes = vec![tri_mesh("p1", "Mat_A"), tri_mesh("p2", "Mat_A")];
    let normals = vec![vec![[0.0, 0.0, 1.0]; 6]; 2];
    export(&out, "T", &[], &meshes, &normals, None).unwrap();
    let text = std::fs::read_to_string(&out).unwrap();
    assert!(text.contains("def Mesh \"Mat_A\""));
    assert!(text.contains("faceVertexCounts = [3, 3]"));
    std::fs::remove_dir_all(&dir).unwrap();
}

/// 有骨骼：按 (关节,材质) 分组命名 Mesh_<关节>_<材质>
#[test]
fn group_by_joint_and_material() {
    let dir = std::env::temp_dir().join(format!("juno2usd-export-skel-{}", std::process::id()));
    let out = dir.join("skel.usda");
    let meshes = vec![tri_mesh("p1", "Mat_A"), tri_mesh("p2", "Mat_A")];
    let normals = vec![vec![[0.0, 0.0, 1.0]; 6]; 2];
    let sk = Skeleton {
        joints: vec![
            Joint {
                path: "joint_j0".into(),
                part_id: "j0".into(),
                bind: identity(),
                rest: identity(),
            },
            Joint {
                path: "joint_j1".into(),
                part_id: "j1".into(),
                bind: identity(),
                rest: identity(),
            },
        ],
        part_joint: [("p1".into(), 0), ("p2".into(), 1)].into_iter().collect(),
    };
    export(&out, "T", &[], &meshes, &normals, Some(&sk)).unwrap();
    let text = std::fs::read_to_string(&out).unwrap();
    assert!(text.contains("Mesh_0_Mat_A"));
    assert!(text.contains("Mesh_1_Mat_A"));
    assert!(text.contains("skel:jointIndices"));
    std::fs::remove_dir_all(&dir).unwrap();
}
