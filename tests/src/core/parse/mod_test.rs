use super::*;

/// 重导出的解析入口与领域类型可用
#[test]
fn reexport_parse_file_runs() {
    let dir = std::env::temp_dir().join(format!("juno2usd-parsemod-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("m.xml");
    std::fs::write(
        &input,
        r#"<Ship><Part id="p" partType="Fuselage1"><Fuselage offset="0,1,0"/></Part></Ship>"#,
    )
    .unwrap();
    let m = parse_file(&input, &crate::settings::Settings::load().unwrap()).unwrap();
    assert_eq!(m.parts.len(), 1);
    assert_eq!(PartType::from_xml("Rotator1"), Some(PartType::Rotator));
    std::fs::remove_dir_all(&dir).unwrap();
}
