use super::*;

fn tri_mesh(normals: Vec<[f32; 3]>) -> CompactMesh {
    CompactMesh {
        points: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
        faces: vec![[0, 1, 2]],
        normals,
    }
}

/// 写入 points/counts/indices/st；法线 faceVarying；无 normals 不写
#[test]
fn write_mesh_attrs_and_facevarying_normals() {
    let (stage, root) = super::super::stage::create("T").unwrap();
    write(
        &stage,
        &format!("/{root}/M"),
        &tri_mesh(vec![[0.0, 0.0, 1.0]; 3]),
    )
    .unwrap();
    let text = stage.root_layer().borrow().export_to_string().unwrap();
    assert!(text.contains("faceVertexCounts = [3]"));
    assert!(text.contains("faceVertexIndices = [0, 1, 2]"));
    assert!(text.contains("faceVarying"));
    assert!(text.contains("texCoord2f[] primvars:st"));
    assert!(text.contains("subdivisionScheme = \"none\""));

    let (stage2, root2) = super::super::stage::create("T").unwrap();
    write(&stage2, &format!("/{root2}/M"), &tri_mesh(Vec::new())).unwrap();
    let text2 = stage2.root_layer().borrow().export_to_string().unwrap();
    assert!(!text2.contains("faceVarying"));
}
