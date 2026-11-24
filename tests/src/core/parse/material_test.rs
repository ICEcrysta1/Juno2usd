use super::*;

/// USD 命名清理
#[test]
fn clean_name_replaces_invalid_chars() {
    assert_eq!(clean_name("Mat A-b.c/d"), "Mat_A_b_c_d");
}

/// hex→RGB、roughness=1-s；DesignerSettings 优先
#[test]
fn theme_material_parsing() {
    let doc = roxmltree::Document::parse(
        r#"<Ship><DesignerSettings><Theme>
             <Material name="Red" color="FF0000" m="0.25" s="0.5"/>
           </Theme></DesignerSettings></Ship>"#,
    )
    .unwrap();
    let ms = parse(&doc);
    assert_eq!(ms.len(), 1);
    assert_eq!(ms[0].color, [1.0, 0.0, 0.0]);
    assert_eq!(ms[0].metallic, 0.25);
    assert_eq!(ms[0].roughness, 0.5);
}

/// 无 Theme 空列表；坏 hex 回退白色
#[test]
fn fallbacks() {
    let doc = roxmltree::Document::parse("<Ship/>").unwrap();
    assert!(parse(&doc).is_empty());
    assert_eq!(parse_hex_color("zz"), [1.0, 1.0, 1.0]);
    assert_eq!(parse_hex_color("00FF00"), [0.0, 1.0, 0.0]);
}
