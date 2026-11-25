use super::*;
use std::f64::consts::PI;

/// r=1 圆 / r=0 方形（边界 max|coord|=1）
#[test]
fn circle_and_square_limits() {
    let (x, z) = rounded_rect_point(0.0, 2.0, 3.0, &[1.0; 4], 0.0, &[1.0; 4]);
    assert!((x - 2.0).abs() < 1e-9 && z.abs() < 1e-9);
    let (x, z) = rounded_rect_point(PI / 2.0, 2.0, 3.0, &[1.0; 4], 0.0, &[1.0; 4]);
    assert!(x.abs() < 1e-9 && (z - 3.0).abs() < 1e-9);
    let (x, z) = rounded_rect_point(PI / 4.0, 1.0, 1.0, &[0.0; 4], 0.0, &[1.0; 4]);
    assert!((x.abs() - 1.0).abs() < 1e-9 && (z.abs() - 1.0).abs() < 1e-9);
}

/// 水滴形变：下半 x 收缩、z 不变
#[test]
fn teardrop_deformation_shrinks_lower_x() {
    let a = 5.0 * PI / 4.0;
    let (x0, z0) = rounded_rect_point(a, 1.0, 1.0, &[1.0; 4], 0.0, &[1.0; 4]);
    let (x1, z1) = rounded_rect_point(a, 1.0, 1.0, &[1.0; 4], 0.5, &[1.0; 4]);
    assert!(x0 < 0.0 && z0 < 0.0);
    assert!(x1 > x0 && (z1 - z0).abs() < 1e-9);
}

/// clamp 在单位空间硬裁剪：超出 ±c 截断到边界再乘半径；>=1 无操作
#[test]
fn clamp_clips_unit_point() {
    let (x, z) = rounded_rect_point(0.0, 2.0, 3.0, &[1.0; 4], 0.0, &[1.0, 0.5, 1.0, 1.0]);
    assert!((x - 1.0).abs() < 1e-9 && z.abs() < 1e-9);
    let (x, _) = rounded_rect_point(PI, 2.0, 3.0, &[1.0; 4], 0.0, &[0.5, 1.0, 1.0, 1.0]);
    assert!((x + 1.0).abs() < 1e-9);
    let (x, z) = rounded_rect_point(0.0, 2.0, 3.0, &[1.0; 4], 0.0, &[1.0; 4]);
    assert!((x - 2.0).abs() < 1e-9 && z.abs() < 1e-9);
}

/// 竖向剪切：z=顶半径 y 不变，z=0 下移 cut_depth/2
#[test]
fn shear_y_profile() {
    assert_eq!(shear_y(5.0, 2.0, 2.0, 4.0), 5.0);
    assert_eq!(shear_y(5.0, 0.0, 2.0, 4.0), 3.0);
    assert_eq!(shear_y(5.0, 1.0, 0.0, 4.0), 3.0);
}

/// 空心径向收缩：(dist-wall)/dist；dist<=wall 归零
#[test]
fn shrink_radial_wall() {
    let p = shrink_radial(2.0, 0.0, 7.0, 0.5);
    assert!((p[0] - 1.5).abs() < 1e-9 && p[1] == 7.0);
    assert_eq!(shrink_radial(0.3, 0.0, 7.0, 0.5), [0.0, 7.0, 0.0]);
}
