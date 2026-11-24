//! 转换核心算法域：parse（XML 解析）/ shapes（几何）/ normal（法线）/ skel（骨骼）/ usda（导出）。
//!
//! 对外编排入口与进度类型后续在此定义；核心同步、无 UI 依赖。

pub mod normal;
pub mod parse;
pub mod shapes;
pub mod skel;
pub mod usda;
