//! 截面算法：圆角矩形（圆↔方）、水滴形变、clamp 挤压、竖向剪切、空心收缩。

use super::transform::Vec3;
use std::f64::consts::PI;

const TAU: f64 = 2.0 * PI;

/// 圆角矩形截面点（支持水滴形变与 clamp 裁剪），返回 (x, z)
///
/// corner_radii 按象限取值：左上/右上/右下/左下；
/// r≈0 为方形（射线求交），r≈1 为圆，中间为圆角矩形。
/// clamp 顺序 x_neg/x_pos/z_neg/z_pos：在单位空间把形变后的坐标硬裁剪到
/// [-|c_neg|, |c_pos|]，再乘半径（裁剪平面相对截面中心，不受 offset/scale 影响）。
pub fn rounded_rect_point(
    angle: f64,
    radius_x: f64,
    radius_z: f64,
    corner_radii: &[f64],
    deformation: f64,
    clamp: &[f64],
) -> (f64, f64) {
    let a = angle.rem_euclid(TAU);

    let (corner_idx, sign_x, sign_z) = if (PI / 2.0..PI).contains(&a) {
        (0, -1.0, 1.0) // 左上
    } else if (0.0..PI / 2.0).contains(&a) {
        (1, 1.0, 1.0) // 右上
    } else if (3.0 * PI / 2.0..TAU).contains(&a) {
        (2, 1.0, -1.0) // 右下
    } else {
        (3, -1.0, -1.0) // 左下
    };

    let r = corner_radii.get(corner_idx).copied().unwrap_or(1.0);

    let (mut ux, mut uz) = if r <= 0.001 {
        square_point(a)
    } else if r >= 0.999 {
        (a.cos(), a.sin())
    } else {
        // 圆角矩形：角圆心 ±(1-r)，按切线角判定弧段/直边
        let cx = sign_x * (1.0 - r);
        let cz = sign_z * (1.0 - r);
        let angle_tangent = (sign_z * (1.0 - r)).atan2(sign_x).rem_euclid(TAU);
        let angle_corner = sign_z.atan2(sign_x).rem_euclid(TAU);

        if angle_diff(a, angle_corner) <= angle_diff(angle_tangent, angle_corner) {
            (cx + r * a.cos(), cz + r * a.sin())
        } else {
            square_point(a)
        }
    };

    // 水滴形变：Z 越低 X 收缩越多
    if deformation > 0.001 {
        let z_norm = uz.clamp(-1.0, 1.0);
        let width_ratio = (z_norm + 1.0) / 2.0;
        ux *= 1.0 - deformation * (1.0 - width_ratio);
    }

    // clamp：单位空间硬裁剪（取绝对值作为上下界，>=1 自然无操作）
    let bound = |i: usize| clamp.get(i).map(|v| v.abs()).unwrap_or(1.0);
    ux = ux.clamp(-bound(0), bound(1));
    uz = uz.clamp(-bound(2), bound(3));

    (radius_x * ux, radius_z * uz)
}

/// 方形截面：射线与单位方形边界求交
fn square_point(a: f64) -> (f64, f64) {
    let abs_cos = a.cos().abs();
    let abs_sin = a.sin().abs();
    let t = if abs_cos < 0.0001 || abs_sin < 0.0001 {
        1.0
    } else {
        (1.0 / abs_cos).min(1.0 / abs_sin)
    };
    (t * a.cos(), t * a.sin())
}

/// 圆周上两角的最短差值
fn angle_diff(a: f64, b: f64) -> f64 {
    let d = (a - b).abs();
    if d > PI { TAU - d } else { d }
}

/// 竖向剪切：cut_depth = 2·offset_y·shear·t（t 由调用方给定）
pub fn shear_y(y: f64, z: f64, rz_scaled: f64, cut_depth: f64) -> f64 {
    let z_norm = if rz_scaled > 0.001 {
        z / rz_scaled
    } else {
        0.0
    };
    y + cut_depth * (z_norm - 1.0) / 2.0
}

/// 空心内环：按径向收缩壁厚，(dist-wall)/dist；收缩到中心以内则归零
pub fn shrink_radial(x: f64, z: f64, y: f64, wall: f64) -> Vec3 {
    let dist = (x * x + z * z).sqrt();
    if dist > wall {
        let ratio = (dist - wall) / dist;
        [x * ratio, y, z * ratio]
    } else {
        [0.0, y, 0.0]
    }
}

#[cfg(test)]
#[path = "../../../../tests/src/core/shapes/algo/section_test.rs"]
mod tests;
