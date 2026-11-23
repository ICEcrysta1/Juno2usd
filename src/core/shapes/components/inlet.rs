//! Inlet1 / FairingBase1 / Fairing1 几何：空心圆柱。
//!
//! 复用 fuselage 的共享圆柱（wall_thickness > 0 → 内环 + 环形端盖）；
//! 端盖朝向等法线策略后续由本模块声明，normal 不识别零件类型。

use super::super::RawMesh;
use crate::core::parse::Part;

pub fn generate(part: &Part, segments: usize) -> Option<RawMesh> {
    super::fuselage::generate(part, segments)
}
