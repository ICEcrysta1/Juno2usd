/// 中文文案键值稳定
#[test]
fn zh_hans_strings() {
    assert_eq!(message!(ui_info), "信息");
    assert_eq!(message!(ui_choose), "选择");
    assert_eq!(message!(ui_parts), "零件");
    assert_eq!(message!(ui_size), "大小");
    assert_eq!(message!(ui_key_quit), "q/Esc 退出");
    assert_eq!(format!(message!(ui_title), "0.2.0"), "Juno2USD 版本: 0.2.0");
}
