//! 共享几何算法（纯函数，无零件语义）。
//!
//! - section：圆角矩形截面（圆↔方）、水滴形变、clamp 裁剪、竖向剪切、空心收缩
//! - ring：环顶点发射、环间侧面缝合（四边面）、端盖拓扑
//! - triangulate：四边面优化三角化（法线计算前）
//! - transform：Unity ZXY 旋转、局部→世界变换、环缩放

pub mod ring;
pub mod section;
pub mod transform;
pub mod triangulate;

#[cfg(test)]
#[path = "../../../../tests/src/core/shapes/algo/mod_test.rs"]
mod tests;
