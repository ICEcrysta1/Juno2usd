//! 骨骼域：从零件连接树构建关节层级与零件绑定（UsdSkel 数据）。

pub mod build;

pub use build::{Skeleton, build};

#[cfg(test)]
#[path = "../../../tests/src/core/skel/mod_test.rs"]
mod tests;
