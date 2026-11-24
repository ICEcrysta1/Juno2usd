//! 编译期文案；静态文字不分配内存，格式化沿用 Rust 的编译期参数检查。
//! 模式同 PadNorm：locales/*.rs 的 message! 宏 + i18n! 分发宏。

use std::sync::atomic::{AtomicU8, Ordering};

#[path = "locales/en.rs"]
pub(crate) mod en;
#[path = "locales/zh-hans.rs"]
pub(crate) mod zh_hans;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    ZhHans,
    En,
}

static LANGUAGE: AtomicU8 = AtomicU8::new(0);

pub fn language() -> Language {
    match LANGUAGE.load(Ordering::Relaxed) {
        0 => Language::ZhHans,
        _ => Language::En,
    }
}

pub fn set_language(language: Language) {
    LANGUAGE.store(language as u8, Ordering::Relaxed);
}

/// 依环境变量（LC_ALL/LANG）初始化，含 zh 为中文，否则英文
pub fn init_from_env() {
    let v = std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default();
    set_language(if v.to_lowercase().contains("zh") {
        Language::ZhHans
    } else {
        Language::En
    });
}

macro_rules! i18n {
    ($key:ident) => {
        match $crate::tui::i18n::language() {
            $crate::tui::i18n::Language::ZhHans => $crate::tui::i18n::zh_hans::message!($key),
            $crate::tui::i18n::Language::En => $crate::tui::i18n::en::message!($key),
        }
    };
    ($key:ident, $($args:tt)*) => {
        match $crate::tui::i18n::language() {
            $crate::tui::i18n::Language::ZhHans => format!($crate::tui::i18n::zh_hans::message!($key), $($args)*),
            $crate::tui::i18n::Language::En => format!($crate::tui::i18n::en::message!($key), $($args)*),
        }
    };
}

pub(crate) use i18n;

#[cfg(test)]
#[path = "../../../tests/src/tui/i18n/i18n_test.rs"]
mod tests;
