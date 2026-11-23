//! 具体零件的几何生成与类型分发。

pub mod fuselage;
pub mod inlet;
pub mod model;
pub mod nose_cone;

use crate::core::parse::{Part, PartType};
use model::RawMesh;

/// 按零件类型分发几何生成
pub fn generate(part: &Part, segments: usize) -> Option<RawMesh> {
    match part.part_type {
        PartType::Fuselage | PartType::Strut => fuselage::generate(part, segments),
        PartType::Inlet | PartType::FairingBase | PartType::Fairing => {
            inlet::generate(part, segments)
        }
        PartType::NoseCone | PartType::FairingNoseCone => nose_cone::generate(part, segments),
        _ => None,
    }
}
