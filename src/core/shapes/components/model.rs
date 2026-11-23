//! 几何数据定义：面分组与零件网格。

use super::super::algo::transform;
use crate::core::parse::Part;

/// 面分组：几何生成时打标，供法线模块消费，无需事后分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FaceGroup {
    OuterSide,
    InnerSide,
    CapBottom,
    CapTop,
}

impl FaceGroup {
    /// 侧面组走平滑组算法；端盖组取平面法线
    pub fn is_side(self) -> bool {
        matches!(self, Self::OuterSide | Self::InnerSide)
    }
}

/// 单个零件的网格（局部空间）。UV 与顶点一一对应，面只索引顶点。
#[derive(Debug, Clone)]
pub struct RawMesh {
    pub part_id: String,
    pub material_name: String,
    pub vertices: Vec<transform::Vec3>,
    pub uvs: Vec<[f64; 2]>,
    pub faces: Vec<[usize; 3]>,
    /// 与 faces 一一对应
    pub face_groups: Vec<FaceGroup>,
    /// 几何声明的锐边（无序顶点对），如鼻锥尖端顶点的所有边
    pub sharp_edges: std::collections::HashSet<(usize, usize)>,
}

impl RawMesh {
    pub(super) fn new(part: &Part) -> Self {
        Self {
            part_id: part.id.clone(),
            material_name: part.material_name.clone(),
            vertices: Vec::new(),
            uvs: Vec::new(),
            faces: Vec::new(),
            face_groups: Vec::new(),
            sharp_edges: std::collections::HashSet::new(),
        }
    }

    pub fn push_vertex(&mut self, v: transform::Vec3, uv: [f64; 2]) -> usize {
        let idx = self.vertices.len();
        self.vertices.push(v);
        self.uvs.push(uv);
        idx
    }

    /// 追加一批面并打上分组标签
    pub fn add_faces(&mut self, faces: Vec<[usize; 3]>, group: FaceGroup) {
        self.face_groups
            .resize(self.faces.len() + faces.len(), group);
        self.faces.extend(faces);
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
