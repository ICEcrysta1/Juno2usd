//! 法线纯数学执行器：消费面分组，输出局部空间 faceVarying 法线。
//!
//! 不识别零件类型；朝向由绕序决定，永不硬编码方向；
//! 世界变换由调用方经 algo::transform 完成。

pub mod cap;
pub mod compute;
pub mod smooth;

pub use compute::compute;
