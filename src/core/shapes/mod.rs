//! 零件几何：共享算法（algo）+ 具体零件生成（components）。
//!
//! 输入 parse 产出的 Part，输出局部空间 RawMesh；
//! 法线策略声明后续归属各零件模块。

pub mod algo;
pub mod components;

pub use components::generate;
pub use components::model::RawMesh;
