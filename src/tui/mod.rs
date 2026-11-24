//! 终端交互界面。

pub mod app;
pub mod controls;
#[path = "i18n/i18n.rs"]
pub mod i18n;
pub mod page;

#[cfg(test)]
#[path = "../../tests/src/tui/mod_test.rs"]
mod tests;
