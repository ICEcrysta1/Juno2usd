use super::*;

// 语言为全局状态：本文件内串行化，避免并行测试互相覆盖
static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// set_language 后 i18n! 分发到对应 locale
#[test]
fn set_language_dispatch() {
    let _g = LOCK.lock().unwrap();
    set_language(Language::En);
    assert_eq!(language(), Language::En);
    assert_eq!(i18n!(ui_info), "Info");
    assert_eq!(i18n!(ui_title, "0.2.0"), "Juno2USD Version: 0.2.0");
    set_language(Language::ZhHans);
    assert_eq!(language(), Language::ZhHans);
    assert_eq!(i18n!(ui_info), "信息");
    assert_eq!(i18n!(ui_title, "0.2.0"), "Juno2USD 版本: 0.2.0");
}

/// LC_ALL/LANG 含 zh → 中文，否则英文
#[test]
fn init_from_env_zh_detection() {
    let _g = LOCK.lock().unwrap();
    unsafe { std::env::set_var("LC_ALL", "zh_CN.UTF-8") };
    init_from_env();
    assert_eq!(language(), Language::ZhHans);
    unsafe { std::env::set_var("LC_ALL", "en_US.UTF-8") };
    init_from_env();
    assert_eq!(language(), Language::En);
    unsafe { std::env::remove_var("LC_ALL") };
    set_language(Language::ZhHans);
}
