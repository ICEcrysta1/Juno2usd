/// 英文文案键值稳定
#[test]
fn english_strings() {
    assert_eq!(message!(ui_info), "Info");
    assert_eq!(message!(ui_choose), "Choose");
    assert_eq!(message!(ui_parts), "Parts");
    assert_eq!(message!(ui_size), "Size");
    assert_eq!(message!(ui_key_quit), "q/Esc quit");
    assert_eq!(
        format!(message!(ui_title), "0.2.0"),
        "Juno2USD Version: 0.2.0"
    );
}
