//! 法线纯数学执行器：忠实 Blender "Smooth by Angle" 自动平滑。
//!
//! 不识别零件类型；朝向由绕序与角度阈值决定；
//! 调用前应先焊接重合顶点（导出端负责）。

pub mod compute;
pub mod smooth;

pub use compute::compute;
