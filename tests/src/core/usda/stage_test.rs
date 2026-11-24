use super::*;

/// 根名清理 / 数字前缀补下划线 / 空名回退；元数据 upAxis=Y、metersPerUnit=1
#[test]
fn root_naming_and_stage_metadata() {
    let (stage, root) = create("My Model").unwrap();
    assert_eq!(root, "My_Model");
    assert!(stage.prim("/My_Model").is_ok());
    let text = stage.root_layer().borrow().export_to_string().unwrap();
    assert!(text.contains("upAxis = \"Y\""));
    assert!(text.contains("metersPerUnit = 1"));
    assert!(text.contains("defaultPrim = \"My_Model\""));

    let (_, root) = create("1abc").unwrap();
    assert_eq!(root, "_1abc");
    let (_, root) = create("").unwrap();
    assert_eq!(root, "Model");
}
