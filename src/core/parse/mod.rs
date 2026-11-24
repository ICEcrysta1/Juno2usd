//! SR2 XML 解析：partType 分发、Fuselage 参数、材质、连接。
//!
//! 产出领域数据（Part / Material / ConnectionTree）供 shapes 几何与 usda 导出；
//! 本模块不做几何生成。

pub mod connection;
pub mod material;
pub mod model;
pub mod part;

pub use model::{Part, PartType};
pub use part::parse_file;

#[cfg(test)]
#[path = "../../../tests/src/core/parse/mod_test.rs"]
mod tests;
