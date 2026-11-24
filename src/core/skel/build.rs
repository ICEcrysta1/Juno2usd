//! 骨骼构建（迁移自 core_old/skeleton_builder）：
//! 关节识别 → 沿零件树找关节父 → 世界/本地绑定矩阵 → 无向图 BFS 零件绑定。

use crate::core::parse::Part;
use crate::core::parse::connection::ConnectionTree;
use crate::core::shapes::algo::transform::{Vec3, mat_mul, rotate_vec, rotation_matrix, transpose};
use std::collections::{HashMap, HashSet, VecDeque};

/// 骨骼关节（拓扑序，path 含层级）
#[derive(Debug, Clone)]
pub struct Joint {
    pub path: String,
    pub part_id: String,
    /// 世界空间绑定矩阵（行优先 4x4）
    pub bind: [f64; 16],
    /// 相对父关节本地矩阵（行优先 4x4）
    pub rest: [f64; 16],
}

#[derive(Debug, Default)]
pub struct Skeleton {
    pub joints: Vec<Joint>,
    /// 零件 → 关节索引（未绑定零件缺席）
    pub part_joint: HashMap<String, usize>,
}

pub fn build(parts: &[Part], tree: &ConnectionTree) -> Skeleton {
    let joint_ids: Vec<String> = parts
        .iter()
        .filter(|p| p.part_type.is_joint())
        .map(|p| p.id.clone())
        .collect();
    let joint_set: HashSet<&str> = joint_ids.iter().map(|s| s.as_str()).collect();
    let part_by_id: HashMap<&str, &Part> = parts.iter().map(|p| (p.id.as_str(), p)).collect();

    // 沿零件树向上找最近关节父
    let mut parent_joint: HashMap<&str, &str> = HashMap::new();
    for j in &joint_ids {
        let mut cur = tree.child_to_parent.get(j.as_str()).map(|s| s.as_str());
        while let Some(c) = cur {
            if joint_set.contains(c) {
                parent_joint.insert(j.as_str(), c);
                break;
            }
            cur = tree.child_to_parent.get(c).map(|s| s.as_str());
        }
    }
    let mut children: HashMap<&str, Vec<&str>> = HashMap::new();
    for j in &joint_ids {
        if let Some(&p) = parent_joint.get(j.as_str()) {
            children.entry(p).or_default().push(j.as_str());
        }
    }
    let roots: Vec<&str> = joint_ids
        .iter()
        .map(|s| s.as_str())
        .filter(|j| !parent_joint.contains_key(j))
        .collect();

    let world: HashMap<&str, ([[f64; 3]; 3], Vec3)> = joint_ids
        .iter()
        .map(|j| {
            let p = part_by_id[j.as_str()];
            (j.as_str(), (rotation_matrix(p.rotation), p.position))
        })
        .collect();

    // 拓扑序 DFS 发射关节
    let mut joints: Vec<Joint> = Vec::new();
    let mut index_of: HashMap<String, usize> = HashMap::new();
    for r in roots {
        emit(r, "", None, &children, &world, &mut joints, &mut index_of);
    }

    // 绑定：关节绑自己；无向图 BFS 绑定可达非关节零件，遇关节停止
    let mut part_joint: HashMap<String, usize> = HashMap::new();
    for j in &joint_ids {
        part_joint.insert(j.clone(), index_of[j]);
    }
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for c in &tree.connections {
        adj.entry(c.part_a.as_str())
            .or_default()
            .push(c.part_b.as_str());
        adj.entry(c.part_b.as_str())
            .or_default()
            .push(c.part_a.as_str());
    }
    for j in &joints {
        let ji = index_of[&j.part_id];
        let start = j.part_id.as_str();
        let mut visited: HashSet<&str> = HashSet::new();
        visited.insert(start);
        let mut q: VecDeque<&str> = VecDeque::new();
        q.push_back(start);
        while let Some(cur) = q.pop_front() {
            if let Some(ns) = adj.get(cur) {
                for &nb in ns {
                    if !visited.insert(nb) || joint_set.contains(nb) {
                        continue;
                    }
                    if !part_joint.contains_key(nb) {
                        part_joint.insert(nb.to_string(), ji);
                        q.push_back(nb);
                    }
                }
            }
        }
    }

    Skeleton { joints, part_joint }
}

#[allow(clippy::too_many_arguments)]
fn emit<'a>(
    j: &'a str,
    parent_path: &str,
    parent_world: Option<([[f64; 3]; 3], Vec3)>,
    children: &HashMap<&'a str, Vec<&'a str>>,
    world: &HashMap<&'a str, ([[f64; 3]; 3], Vec3)>,
    out: &mut Vec<Joint>,
    index: &mut HashMap<String, usize>,
) {
    let (r, t) = world[j];
    let path = if parent_path.is_empty() {
        format!("joint_{j}")
    } else {
        format!("{parent_path}/joint_{j}")
    };
    let bind = row_major(r, t);
    let rest = match parent_world {
        Some((rp, tp)) => {
            let rpt = transpose(rp);
            let rl = mat_mul(rpt, r);
            let tl = rotate_vec(&rpt, [t[0] - tp[0], t[1] - tp[1], t[2] - tp[2]]);
            row_major(rl, tl)
        }
        None => bind,
    };
    index.insert(j.to_string(), out.len());
    out.push(Joint {
        path: path.clone(),
        part_id: j.to_string(),
        bind,
        rest,
    });
    if let Some(cs) = children.get(j) {
        for &c in cs {
            emit(c, &path, Some((r, t)), children, world, out, index);
        }
    }
}

/// 行优先存储、行向量约定（v·M）：旋转块取 Rᵀ，平移在 [12..14]。
/// 几何世界变换为 w=R·v+t（列向量），等价行向量矩阵 M=Cᵀ。
fn row_major(r: [[f64; 3]; 3], t: Vec3) -> [f64; 16] {
    [
        r[0][0], r[1][0], r[2][0], 0.0, //
        r[0][1], r[1][1], r[2][1], 0.0, //
        r[0][2], r[1][2], r[2][2], 0.0, //
        t[0], t[1], t[2], 1.0,
    ]
}

#[cfg(test)]
#[path = "../../../tests/src/core/skel/build_test.rs"]
mod tests;
