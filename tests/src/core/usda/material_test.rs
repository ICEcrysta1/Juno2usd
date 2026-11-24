use super::*;

/// UsdPreviewSurface 三参数 + 绑定；无数据空材质仍绑定
#[test]
fn preview_surface_and_binding() {
    let (stage, root) = super::super::stage::create("T").unwrap();
    stage
        .define_typed_prim(format!("/{root}/M"), "Mesh")
        .unwrap();
    let data = crate::core::parse::material::Material {
        name: "Red".into(),
        color: [1.0, 0.0, 0.0],
        metallic: 0.25,
        roughness: 0.5,
    };
    define_and_bind(&stage, &root, "Red", Some(&data), &format!("/{root}/M")).unwrap();
    let text = stage.root_layer().borrow().export_to_string().unwrap();
    assert!(text.contains("UsdPreviewSurface"));
    assert!(text.contains("diffuseColor"));
    assert!(text.contains("metallic"));
    assert!(text.contains("roughness"));
    assert!(text.contains("material:binding"));

    let (stage2, root2) = super::super::stage::create("T").unwrap();
    stage2
        .define_typed_prim(format!("/{root2}/M"), "Mesh")
        .unwrap();
    define_and_bind(&stage2, &root2, "Empty", None, &format!("/{root2}/M")).unwrap();
    let text2 = stage2.root_layer().borrow().export_to_string().unwrap();
    assert!(!text2.contains("UsdPreviewSurface"));
    assert!(text2.contains("material:binding"));
}
