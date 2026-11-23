//! 几何数据定义：环拓扑与零件网格。

use super::super::algo::transform;
use crate::core::parse::Part;

/// 环拓扑：供法线模块做面分类，替代旧代码的无类型 ring_info 字典
#[allow(dead_code, reason = "normal 模块落地后消费环拓扑做面分类")]
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
    pub vertices: Vec<transform::Vec3>,
    pub uvs: Vec<[f64; 2]>,
    pub faces: Vec<[usize; 3]>,
    pub rings: Rings,
}

impl RawMesh {
    pub(super) fn new(part: &Part) -> Self {
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

    pub fn push_vertex(&mut self, v: transform::Vec3, uv: [f64; 2]) -> usize {
        let idx = self.vertices.len();
        self.vertices.push(v);
        self.uvs.push(uv);
        idx
    }

    /// 局部 → 世界：R·v + position
    pub fn to_world(&self, part: &Part) -> RawMesh {
        let rot = transform::rotation_matrix(part.rotation);
        let mut out = self.clone();
        for v in out.vertices.iter_mut() {
            *v = transform::to_world(&rot, part.position, *v);
        }
        out
    }
}
