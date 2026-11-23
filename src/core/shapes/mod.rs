//! 零件注册表与 XML 分发入口。
//!
//! 本模块是每类零件的定义中心：参数解析、几何生成、法线策略
//! 均归属各零件模块；normal 只做纯数学执行，usda 只做导出。

pub mod base;
pub mod connection;
pub mod fuselage;
pub mod inlet;
pub mod material;
pub mod nose_cone;
pub mod section;

use crate::settings::Settings;
use base::Vec3;
use roxmltree::Node;
use std::path::Path;

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
    pub fn is_joint(self) -> bool {
        matches!(self, Self::Rotator | Self::HingeRotator)
    }
}

#[derive(Debug, Clone)]
pub struct Part {
    pub id: String,
    pub part_type: PartType,
    pub position: Vec3,
    /// Unity 欧拉角（度数，ZXY）
    pub rotation: Vec3,
    pub material_index: usize,
    pub material_name: String,
    pub fuselage: Option<fuselage::FuselageParams>,
    /// 空心件壁厚（实心为 0）
    pub wall_thickness: f64,
    /// 鼻锥轴向细分
    pub subdivisions: usize,
}

/// 环拓扑：供法线模块做面分类，替代旧代码的无类型 ring_info 字典
#[derive(Debug, Clone)]
pub enum Rings {
    Cylinder {
        bottom: Vec<usize>,
        top: Vec<usize>,
        bottom_inner: Option<Vec<usize>>,
        top_inner: Option<Vec<usize>>,
        bottom_corners: [f64; 4],
        top_corners: [f64; 4],
        bottom_is_point: bool,
        top_is_point: bool,
    },
    NoseCone {
        rings: Vec<Vec<usize>>,
        inner_rings: Option<Vec<Vec<usize>>>,
        has_bottom_cap: bool,
    },
}

/// 单个零件的网格（局部空间）。UV 与顶点一一对应，面只索引顶点。
#[derive(Debug, Clone)]
pub struct RawMesh {
    pub part_id: String,
    pub material_name: String,
    pub vertices: Vec<Vec3>,
    pub uvs: Vec<[f64; 2]>,
    pub faces: Vec<[usize; 3]>,
    pub rings: Rings,
}

impl RawMesh {
    fn new(part: &Part) -> Self {
        Self {
            part_id: part.id.clone(),
            material_name: part.material_name.clone(),
            vertices: Vec::new(),
            uvs: Vec::new(),
            faces: Vec::new(),
            rings: Rings::Cylinder {
                bottom: Vec::new(),
                top: Vec::new(),
                bottom_inner: None,
                top_inner: None,
                bottom_corners: [1.0; 4],
                top_corners: [1.0; 4],
                bottom_is_point: false,
                top_is_point: false,
            },
        }
    }

    pub fn push_vertex(&mut self, v: Vec3, uv: [f64; 2]) -> usize {
        let idx = self.vertices.len();
        self.vertices.push(v);
        self.uvs.push(uv);
        idx
    }

    /// 局部 → 世界：R·v + position
    pub fn to_world(&self, part: &Part) -> RawMesh {
        let rot = base::rotation_matrix(part.rotation);
        let mut out = self.clone();
        for v in out.vertices.iter_mut() {
            *v = base::to_world(&rot, part.position, *v);
        }
        out
    }
}

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

pub struct ParsedModel {
    pub materials: Vec<material::Material>,
    pub parts: Vec<Part>,
    pub connections: connection::ConnectionTree,
}

/// 解析 SR2 XML：材质 + 零件 + 连接
pub fn parse_file(path: &Path, settings: &Settings) -> Result<ParsedModel, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("读取失败: {e}"))?;
    let doc = roxmltree::Document::parse(&text).map_err(|e| format!("XML 解析失败: {e}"))?;

    let materials = material::parse(&doc);
    let parts = doc
        .descendants()
        .filter(|n| n.has_tag_name("Part"))
        .filter_map(|elem| parse_part(elem, &materials, settings))
        .collect();
    let connections = connection::parse(&doc);

    Ok(ParsedModel {
        materials,
        parts,
        connections,
    })
}

fn parse_part(elem: Node, materials: &[material::Material], settings: &Settings) -> Option<Part> {
    let part_type = PartType::from_xml(elem.attribute("partType").unwrap_or("unknown"))?;
    let id = elem.attribute("id").unwrap_or("unknown").to_string();
    let position = elem
        .attribute("position")
        .and_then(parse_vec3)
        .unwrap_or([0.0; 3]);
    let rotation = elem
        .attribute("rotation")
        .and_then(parse_vec3)
        .unwrap_or([0.0; 3]);

    let material_index = elem
        .attribute("materials")
        .unwrap_or("0,0,0,0,0")
        .split(',')
        .next()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);
    let material_name = materials
        .get(material_index)
        .map(|m| material::clean_name(&m.name))
        .unwrap_or_else(|| "default".to_string());

    let mut part = Part {
        id,
        part_type,
        position,
        rotation,
        material_index,
        material_name,
        fuselage: None,
        wall_thickness: 0.0,
        subdivisions: settings.mesh.nose_cone_subdivisions,
    };

    match part_type {
        PartType::Fuselage | PartType::Strut | PartType::NoseCone => {
            part.fuselage = fuselage::parse_params(elem, settings, part_type);
        }
        PartType::Inlet | PartType::FairingBase | PartType::Fairing => {
            part.fuselage = fuselage::parse_params(elem, settings, part_type);
            part.wall_thickness = settings.hollow.wall_thickness;
        }
        PartType::FairingNoseCone => {
            part.fuselage = fuselage::parse_params(elem, settings, part_type);
            part.wall_thickness = settings.hollow.wall_thickness;
        }
        PartType::Rotator | PartType::HingeRotator => {}
    }

    Some(part)
}

/// 解析逗号分隔的三维向量
pub(crate) fn parse_vec3(text: &str) -> Option<Vec3> {
    let v = parse_floats(text);
    if v.len() == 3 {
        Some([v[0], v[1], v[2]])
    } else {
        None
    }
}

/// 解析逗号分隔的浮点列表（跳过非法项）
pub(crate) fn parse_floats(text: &str) -> Vec<f64> {
    text.split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect()
}
