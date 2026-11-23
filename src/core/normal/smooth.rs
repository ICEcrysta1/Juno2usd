//! 平滑法线：绕顶点面扇连通块 + 锐边拆分 + 面积加权平均。
//!
//! 对每个顶点，将其邻面按"共享该顶点入射边"连成面扇，锐边处断开，
//! 每个连通块内面积加权平均（等价 Blender 标记锐边的拆边语义）：
//! - 尖端顶点所有边标锐 → 闭合面扇逐面断开 → 星芒收尖；
//! - 扇面底环处仅扇面间为锐边，经环带平滑边重新连通 → 无断层；
//! - 方形角点两侧面被锐边分开 → 轴对齐硬边。

use crate::core::shapes::algo::transform::{Vec3, cross, dot, normalize};
use std::collections::{HashMap, HashSet};

/// 计算一组侧面的 faceVarying 法线，输出长度 = 3 × faces.len()。
/// sharp_edges 为几何声明的锐边（无序顶点对）。
pub fn smooth_normals(
    vertices: &[Vec3],
    faces: &[[usize; 3]],
    threshold_deg: f64,
    sharp_edges: &HashSet<(usize, usize)>,
) -> Vec<Vec3> {
    let n = faces.len();
    if n == 0 {
        return Vec::new();
    }

    // 未归一化面法线（模长 = 2×面积，天然面积加权）
    let area_normals: Vec<Vec3> = faces.iter().map(|f| face_cross(vertices, f)).collect();
    let unit: Vec<Vec3> = area_normals.iter().map(|&c| normalize(c)).collect();
    let valid: Vec<bool> = area_normals.iter().map(|&c| dot(c, c) > 1e-24).collect();

    // 顶点 → 面 邻接
    let mut vert_faces: HashMap<usize, Vec<usize>> = HashMap::new();
    for (fi, f) in faces.iter().enumerate() {
        for &v in f {
            vert_faces.entry(v).or_default().push(fi);
        }
    }

    // 自动锐边：相邻面二面角超阈值（仅有效面之间）
    let cos_thresh = threshold_deg.to_radians().cos();
    let mut edge_faces: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for (fi, f) in faces.iter().enumerate() {
        for e in edges(f) {
            edge_faces.entry(e).or_default().push(fi);
        }
    }
    let mut sharp = sharp_edges.clone();
    for (e, adj) in &edge_faces {
        if adj.len() == 2
            && valid[adj[0]]
            && valid[adj[1]]
            && dot(unit[adj[0]], unit[adj[1]]) < cos_thresh
        {
            sharp.insert(*e);
        }
    }

    // 逐顶点：面扇按非锐边连通分块，块内面积加权平均
    let mut out = vec![[0.0; 3]; n * 3];
    for (&v, fs) in &vert_faces {
        // 并查集：共享 v 入射边且非锐边的面连通
        let mut parent: HashMap<usize, usize> = fs.iter().map(|&f| (f, f)).collect();
        fn find(parent: &mut HashMap<usize, usize>, x: usize) -> usize {
            let mut r = x;
            while parent[&r] != r {
                r = parent[&r];
            }
            let mut cur = x;
            while parent[&cur] != r {
                let next = parent[&cur];
                parent.insert(cur, r);
                cur = next;
            }
            r
        }
        for (ai, &fa) in fs.iter().enumerate() {
            for &fb in &fs[ai + 1..] {
                if let Some(e) = shared_edge_at(&faces[fa], &faces[fb], v)
                    && !sharp.contains(&e)
                {
                    let (ra, rb) = (find(&mut parent, fa), find(&mut parent, fb));
                    if ra != rb {
                        parent.insert(ra, rb);
                    }
                }
            }
        }

        // 块内累加（仅有效面贡献权重）
        let mut acc: HashMap<usize, Vec3> = HashMap::new();
        for &fi in fs {
            if valid[fi] {
                let root = find(&mut parent, fi);
                let a = acc.entry(root).or_insert([0.0; 3]);
                a[0] += area_normals[fi][0];
                a[1] += area_normals[fi][1];
                a[2] += area_normals[fi][2];
            }
        }

        // 写回每个面角
        for &fi in fs {
            let root = find(&mut parent, fi);
            let nrm = acc.get(&root).copied().map(normalize).unwrap_or([0.0; 3]);
            let nrm = if dot(nrm, nrm) > 0.5 {
                nrm
            } else if valid[fi] {
                unit[fi]
            } else {
                [0.0, 1.0, 0.0]
            };
            // fi 中顶点 v 的所有出现位置（焊接后可能重复索引）
            for k in 0..3 {
                if faces[fi][k] == v {
                    out[fi * 3 + k] = nrm;
                }
            }
        }
    }
    out
}

/// 两面在顶点 v 处的共享边（无序顶点对）
fn shared_edge_at(f: &[usize; 3], g: &[usize; 3], v: usize) -> Option<(usize, usize)> {
    for a in f {
        if *a == v {
            continue;
        }
        for b in g {
            if *b != v && *a == *b {
                return Some((v.min(*a), v.max(*a)));
            }
        }
    }
    None
}

fn edges(f: &[usize; 3]) -> [(usize, usize); 3] {
    [
        (f[0].min(f[1]), f[0].max(f[1])),
        (f[1].min(f[2]), f[1].max(f[2])),
        (f[2].min(f[0]), f[2].max(f[0])),
    ]
}

fn face_cross(vertices: &[Vec3], f: &[usize; 3]) -> Vec3 {
    let p1 = vertices[f[0]];
    let p2 = vertices[f[1]];
    let p3 = vertices[f[2]];
    cross(
        [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]],
        [p3[0] - p1[0], p3[1] - p1[1], p3[2] - p1[2]],
    )
}
