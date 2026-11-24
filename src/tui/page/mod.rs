//! 页面组合：三框 + 底部提示。

mod layout;

pub use layout::Page;

#[cfg(test)]
#[path = "../../../tests/src/tui/page/mod_test.rs"]
mod tests;
