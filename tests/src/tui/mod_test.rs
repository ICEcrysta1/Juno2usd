use super::*;

/// 子模块接线：app 入口 / controls / page / i18n 可达
#[test]
fn tui_modules_wired() {
    let _: fn() -> std::io::Result<()> = app::run;
    let _ = controls::Button::new("x");
    let _: Option<page::Page<'_>> = None;
    assert!(matches!(
        i18n::language(),
        i18n::Language::ZhHans | i18n::Language::En
    ));
}
