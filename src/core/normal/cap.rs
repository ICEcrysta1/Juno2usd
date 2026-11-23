//! 端盖平面法线：组内面法线面积加权平均，朝向由绕序决定。
//!
//! 端盖按构造为平面（含剪切斜盖），整组共享一个法线；
//! 不硬编码 ±Y，旋转/剪切后依然正确。

use crate::core::shapes::algo::transform::{Vec3, cross, dot, normalize};

pub fn cap_normal(vertices: &[Vec3], faces: &[[usize; 3]]) -> Vec3 {
    let mut acc = [0.0; 3];
    for f in faces {
        let p1 = vertices[f[0]];
        let p2 = vertices[f[1]];
        let p3 = vertices[f[2]];
        let c = cross(
            [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]],
            [p3[0] - p1[0], p3[1] - p1[1], p3[2] - p1[2]],
        );
        acc[0] += c[0];
        acc[1] += c[1];
        acc[2] += c[2];
    }
    let n = normalize(acc);
    if dot(n, n) > 0.5 { n } else { [0.0, 1.0, 0.0] }
}
