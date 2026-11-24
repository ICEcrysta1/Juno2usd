use super::*;
use crate::core::shapes::algo::transform::face_normal;
use std::collections::HashSet;

/// 折面二三角：小角平滑（面角≠面法线）、大角锐（=面法线）
#[test]
fn angle_threshold_smooth_vs_sharp() {
    let verts = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.2],
        [0.0, -1.0, 0.2],
    ];
    let faces = vec![[0, 1, 2], [1, 0, 3]];
    let ee: HashSet<(usize, usize)> = HashSet::new();
    let ev: HashSet<usize> = HashSet::new();
    let ns = smooth_normals(&verts, &faces, 60.0, &ee, &ee, &ev);
    let f1 = face_normal(&verts, &faces[0]);
    // 共享边 (0,1) 平滑 → 顶点0处法线被平均（≈+Z），偏离面法线
    assert!(dot(ns[0], f1) < 0.999, "小角应平滑: {:?}", ns[0]);

    let sharp_verts = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 10.0],
        [0.0, -1.0, 10.0],
    ];
    let ns = smooth_normals(&sharp_verts, &faces, 60.0, &ee, &ee, &ev);
    let f1 = face_normal(&sharp_verts, &faces[0]);
    assert!(dot(ns[0], f1) > 0.999, "大角应锐边");
}

/// 边界边自动锐；强制 smooth/sharp 覆盖角度；hard_vertices 仅该顶点硬角
#[test]
fn overrides_and_hard_vertex() {
    let verts = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.2],
        [0.0, -1.0, 0.2],
    ];
    let faces = vec![[0, 1, 2], [1, 0, 3]];
    let ee: HashSet<(usize, usize)> = HashSet::new();
    let ev: HashSet<usize> = HashSet::new();

    // 单面：边界边 → 全硬
    let ns = smooth_normals(&verts, &faces[..1], 60.0, &ee, &ee, &ev);
    assert!(dot(ns[0], face_normal(&verts, &faces[0])) > 0.999);

    // 大角几何 + 强制平滑边 → 平均
    let sharp_verts = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 10.0],
        [0.0, -1.0, 10.0],
    ];
    let smooth_edges: HashSet<(usize, usize)> = [(0, 1)].into_iter().collect();
    let ns = smooth_normals(&sharp_verts, &faces, 60.0, &smooth_edges, &ee, &ev);
    assert!(dot(ns[0], face_normal(&sharp_verts, &faces[0])) < 0.999);

    // 小角几何 + 强制锐边 → 硬
    let sharp_edges: HashSet<(usize, usize)> = [(0, 1)].into_iter().collect();
    let ns = smooth_normals(&verts, &faces, 60.0, &ee, &sharp_edges, &ev);
    assert!(dot(ns[0], face_normal(&verts, &faces[0])) > 0.999);

    // 硬点顶点0：该顶点硬角，顶点1仍平滑
    let hard: HashSet<usize> = [0].into_iter().collect();
    let ns = smooth_normals(&verts, &faces, 60.0, &ee, &ee, &hard);
    assert!(dot(ns[0], face_normal(&verts, &faces[0])) > 0.999);
    assert!(dot(ns[1], face_normal(&verts, &faces[0])) < 0.999);
}

/// 空面 → 空输出
#[test]
fn empty_faces_empty_normals() {
    let ee: HashSet<(usize, usize)> = HashSet::new();
    let ev: HashSet<usize> = HashSet::new();
    assert!(smooth_normals(&[], &[], 60.0, &ee, &ee, &ev).is_empty());
}
