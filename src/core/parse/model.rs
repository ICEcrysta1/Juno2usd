//! 解析产出的领域数据定义。

use crate::core::shapes::algo::transform::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartType {
    Fuselage,
    Strut,
    Inlet,
    FairingBase,
    Fairing,
    NoseCone,
    FairingNoseCone,
    Rotator,
    HingeRotator,
}

impl PartType {
    pub fn from_xml(s: &str) -> Option<Self> {
        match s {
            "Fuselage1" => Some(Self::Fuselage),
            "Strut1" => Some(Self::Strut),
            "Inlet1" => Some(Self::Inlet),
            "FairingBase1" => Some(Self::FairingBase),
            "Fairing1" => Some(Self::Fairing),
            "NoseCone1" => Some(Self::NoseCone),
            "FairingNoseCone1" => Some(Self::FairingNoseCone),
            "Rotator1" => Some(Self::Rotator),
            "HingeRotator1" => Some(Self::HingeRotator),
            _ => None,
        }
    }

    /// 关节零件：无几何，只参与骨骼
    #[allow(dead_code, reason = "骨骼模块落地后消费")]
    pub fn is_joint(self) -> bool {
        matches!(self, Self::Rotator | Self::HingeRotator)
    }
}

/// 端部平滑开关（XML flattenNormals）。
/// 游戏语义（与字面相反）：被覆盖的端 = 开平滑（参与接头法线匹配），缺省 = 关。
/// 接头规则：双开互相平均；单开则开侧法线对齐关侧；双关不平滑。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlattenNormals {
    #[default]
    None,
    Top,
    Bottom,
    Both,
}

impl FlattenNormals {
    pub fn from_xml(s: &str) -> Self {
        match s {
            "Top" => Self::Top,
            "Bottom" => Self::Bottom,
            "Both" => Self::Both,
            _ => Self::None,
        }
    }

    /// 底端是否开平滑
    pub fn smooth_bottom(self) -> bool {
        matches!(self, Self::Bottom | Self::Both)
    }

    /// 顶端是否开平滑
    pub fn smooth_top(self) -> bool {
        matches!(self, Self::Top | Self::Both)
    }
}

/// Fuselage 参数（语义与 SR2 XML 一一对应）
#[derive(Debug, Clone)]
pub struct FuselageParams {
    /// 长度 = offset_y × 2
    pub length: f64,
    pub radius_x: f64,
    pub radius_z: f64,
    pub offset: Vec3,
    /// (x, z)
    pub top_scale: [f64; 2],
    pub bottom_scale: [f64; 2],
    /// (顶部水滴, 竖向剪切, 底部水滴)
    pub deformations: [f64; 3],
    /// 顶4 + 底4，象限序：左上/右上/右下/左下
    pub corner_radiuses: [f64; 8],
    /// tx-,tx+,tz-,tz+,bx-,bx+,bz-,bz+
    pub clamp_distances: [f64; 8],
    /// 整体额外缩放，旋转前应用
    pub part_scale: Vec3,
    /// 端部平滑开关（门控接头法线混合）
    pub flatten_normals: FlattenNormals,
}

#[derive(Debug, Clone)]
pub struct Part {
    pub id: String,
    pub part_type: PartType,
    pub position: Vec3,
    /// Unity 欧拉角（度数，ZXY）
    pub rotation: Vec3,
    #[allow(dead_code, reason = "材质索引语义保留，骨骼/导出后续消费")]
    pub material_index: usize,
    pub material_name: String,
    pub fuselage: Option<FuselageParams>,
    /// 空心件壁厚（实心为 0）
    pub wall_thickness: f64,
    /// 鼻锥轴向细分
    pub subdivisions: usize,
}

pub struct ParsedModel {
    pub materials: Vec<super::material::Material>,
    pub parts: Vec<Part>,
    pub connections: super::connection::ConnectionTree,
}

#[cfg(test)]
#[path = "../../../tests/src/core/parse/model_test.rs"]
mod tests;
