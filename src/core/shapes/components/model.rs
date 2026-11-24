//! 几何数据定义：零件网格。

use super::super::algo::transform;
use crate::core::parse::Part;

/// 面二分：侧面走自动平滑（焊接后零件间光滑），端盖单独算平面法线（硬边）。
/// 二者必须分 pass：焊接对接环边被侧+盖共 4 面共享，统一 pass 会判非流形锐边而拆开侧面。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceGroup {
    Side,
    Cap,
}

/// 端环：顶点索引环 + 该端是否开平滑（flattenNormals 覆盖）。
/// 供接头法线混合定位对接环并读取开关。
#[derive(Debug, Clone)]
pub struct Rim {
    pub vertices: Vec<usize>,
    pub smooth: bool,
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
    /// 强制平滑边（无序顶点对）：鼻锥环向等，覆盖角度判定
    pub smooth_edges: std::collections::HashSet<(usize, usize)>,
    /// 强制锐边（无序顶点对）：方形截面纵向棱等，覆盖角度判定保持硬边
    pub sharp_edges: std::collections::HashSet<(usize, usize)>,
    /// 尖点顶点（圆锥尖）：其入射边强制锐边
    pub apex_vertices: std::collections::HashSet<usize>,
    /// 硬点顶点（非圆锥缩 0 端）：仅该顶点面角取面法线（硬角），入射边不标锐→锥面仍光滑
    pub hard_vertices: std::collections::HashSet<usize>,
    /// 端环：供接头法线混合定位对接环
    pub rims: Vec<Rim>,
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
            smooth_edges: std::collections::HashSet::new(),
            sharp_edges: std::collections::HashSet::new(),
            apex_vertices: std::collections::HashSet::new(),
            hard_vertices: std::collections::HashSet::new(),
            rims: Vec::new(),
        }
    }

    pub fn push_vertex(&mut self, v: transform::Vec3, uv: [f64; 2]) -> usize {
        let idx = self.vertices.len();
        self.vertices.push(v);
        self.uvs.push(uv);
        idx
    }

    /// 追加一批面并打标签
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
