//! 平滑法线：忠实 Blender "Smooth by Angle"（自动平滑）算法。
//!
//! 参考 Blender `bmesh_mesh_normals.cc`：
//! 1. 标锐边：边界边（1 面）/ 非流形边（>2 面）自动锐；恰好 2 面则法线夹角 > 阈值 → 锐
//! 2. 绕顶点将邻面按非锐边连成 smooth fan
//! 3. 扇内按顶点处 corner angle 加权平均面法线，赋给每个面角（split normal）
//!
//! 逐零件计算；跨零件接头平滑由 normal::blend 单独处理。

use crate::core::shapes::algo::transform::{Vec3, dot, face_cross, normalize};
use std::collections::HashMap;

/// 计算 faceVarying 法线，输出长度 = 3 × faces.len()。
/// smooth_edges 为强制平滑边（跨零件界面），sharp_edges 为强制锐边（圆锥尖入射边），
/// 二者均覆盖角度判定；sharp 优先。hard_vertices 为硬点顶点：仅该顶点面角取面法线，
/// 其入射边在其他顶点处仍正常平滑（缩 0 端硬点但锥面光滑）。
pub fn smooth_normals(
    vertices: &[Vec3],
    faces: &[[usize; 3]],
    threshold_deg: f64,
    smooth_edges: &std::collections::HashSet<(usize, usize)>,
    sharp_edges: &std::collections::HashSet<(usize, usize)>,
    hard_vertices: &std::collections::HashSet<usize>,
) -> Vec<Vec3> {
    let n = faces.len();
    if n == 0 {
        return Vec::new();
    }

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

    // 边 → 面 邻接
    let mut edge_faces: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for (fi, f) in faces.iter().enumerate() {
        for e in edges(f) {
            edge_faces.entry(e).or_default().push(fi);
        }
    }

    // 标锐边：边界/非流形自动锐；2 面按角度阈值
    let cos_thresh = threshold_deg.to_radians().cos();
    let mut sharp: HashMap<(usize, usize), bool> = HashMap::new();
    for (e, adj) in &edge_faces {
        let is_sharp = if sharp_edges.contains(e) {
            true
        } else if smooth_edges.contains(e) {
            false
        } else if adj.len() != 2 {
            true
        } else {
            dot(unit[adj[0]], unit[adj[1]]) < cos_thresh
        };
        sharp.insert(*e, is_sharp);
    }

    // 逐顶点：smooth fan 连通块 + corner angle 加权平均
    let mut out = vec![[0.0; 3]; n * 3];
    for (&v, fs) in &vert_faces {
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
        // 硬点顶点：不合并扇面 → 每个面角取自身面法线（硬角）；入射边在其他顶点仍平滑
        if !hard_vertices.contains(&v) {
            for (ai, &fa) in fs.iter().enumerate() {
                for &fb in &fs[ai + 1..] {
                    if let Some(e) = shared_edge_at(&faces[fa], &faces[fb], v)
                        && !sharp.get(&e).copied().unwrap_or(true)
                    {
                        let (ra, rb) = (find(&mut parent, fa), find(&mut parent, fb));
                        if ra != rb {
                            parent.insert(ra, rb);
                        }
                    }
                }
            }
        }

        // 块内按 corner angle 加权累加（仅有效面贡献）
        let mut acc: HashMap<usize, Vec3> = HashMap::new();
        for &fi in fs {
            if !valid[fi] {
                continue;
            }
            let w = corner_angle(vertices, &faces[fi], v);
            let root = find(&mut parent, fi);
            let a = acc.entry(root).or_insert([0.0; 3]);
            a[0] += w * unit[fi][0];
            a[1] += w * unit[fi][1];
            a[2] += w * unit[fi][2];
        }

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
            for k in 0..3 {
                if faces[fi][k] == v {
                    out[fi * 3 + k] = nrm;
                }
            }
        }
    }
    out
}

/// 面 f 在顶点 v 处的 corner angle（两条入射边夹角）
fn corner_angle(vertices: &[Vec3], f: &[usize; 3], v: usize) -> f64 {
    let p = vertices[v];
    let mut dirs = Vec::with_capacity(2);
    for &idx in f {
        if idx != v {
            let q = vertices[idx];
            dirs.push(normalize([q[0] - p[0], q[1] - p[1], q[2] - p[2]]));
        }
    }
    if dirs.len() == 2 {
        dot(dirs[0], dirs[1]).clamp(-1.0, 1.0).acos()
    } else {
        0.0
    }
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
