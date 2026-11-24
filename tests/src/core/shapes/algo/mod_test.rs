use super::*;

/// 四个共享算法模块接线可用
#[test]
fn algo_modules_wired() {
    assert_eq!(transform::normalize([0.0; 3]), [0.0; 3]);
    let (x, z) = section::rounded_rect_point(0.0, 1.0, 1.0, &[1.0; 4], 0.0);
    assert!((x - 1.0).abs() < 1e-9 && z.abs() < 1e-9);
    assert_eq!(ring::perimeter_edges(&[0, 1]), vec![(0, 1), (0, 1)]);
    assert_eq!(
        transform::transpose(transform::rotation_matrix([0.0; 3])),
        transform::rotation_matrix([0.0; 3])
    );
}
