use super::*;

/// 重导出的编排入口可跑通最小转换
#[test]
fn reexport_convert_runs() {
    let dir = std::env::temp_dir().join(format!("juno2usd-coremod-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("m.xml");
    std::fs::write(
        &input,
        r#"<Ship><Part id="p" partType="Fuselage1"><Fuselage offset="0,1,0"/></Part></Ship>"#,
    )
    .unwrap();
    let r = convert(&input, None, None, |_| {}).unwrap();
    assert_eq!(r.parts, 1);
    assert!(r.faces > 0);
    std::fs::remove_dir_all(&dir).unwrap();
}
