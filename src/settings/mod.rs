//! 运行参数加载：YAML 为权威。
//!
//! 影响算法、数据语义与可复现行为的参数来自 defaults.yaml；
//! 机器本地路径、凭据等由根目录 .env 经 python-dotenv 管理，不在此处。

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub mesh: Mesh,
    pub fuselage: Fuselage,
    pub hollow: Hollow,
    #[allow(dead_code, reason = "normal 模块落地后消费锐边阈值")]
    pub normals: Normals,
}

#[derive(Debug, Deserialize)]
pub struct Mesh {
    /// 截面周向分段数
    pub segments: usize,
    /// 鼻锥轴向细分段数
    pub nose_cone_subdivisions: usize,
}

#[derive(Debug, Deserialize)]
pub struct Fuselage {
    pub default_radius_x: f64,
    pub default_radius_z: f64,
    /// Strut1 缺省圆角半径
    pub strut_corner_radius: f64,
}

#[derive(Debug, Deserialize)]
pub struct Hollow {
    /// Inlet/Fairing 默认壁厚
    pub wall_thickness: f64,
}

#[derive(Debug, Deserialize)]
pub struct Normals {
    /// 锐边判定阈值（度）
    #[allow(dead_code, reason = "normal 模块落地后消费锐边阈值")]
    pub sharp_edge_angle: f64,
}

impl Settings {
    /// 加载内嵌的权威默认参数
    pub fn load() -> Result<Self, String> {
        serde_saphyr::from_str(include_str!("defaults.yaml")).map_err(|e| e.to_string())
    }
}
