use super::*;

fn close(a: Vec3, b: Vec3) -> bool {
    dist(a, b) < 1e-9
}

/// 零欧拉角 = 单位阵；Y90° 列向量约定 x→-z
#[test]
fn rotation_matrix_unity_zxy() {
    let i = rotation_matrix([0.0; 3]);
    assert!(close(rotate_vec(&i, [1.0, 2.0, 3.0]), [1.0, 2.0, 3.0]));
    let ry90 = rotation_matrix([0.0, 90.0, 0.0]);
    assert!(close(rotate_vec(&ry90, [1.0, 0.0, 0.0]), [0.0, 0.0, -1.0]));
}

/// 矩阵乘单位律 / 转置对合 / 世界变换 = 旋转 + 平移
#[test]
fn mat_mul_transpose_to_world() {
    let m = rotation_matrix([10.0, 20.0, 30.0]);
    let i = rotation_matrix([0.0; 3]);
    assert_eq!(mat_mul(m, i), m);
    assert_eq!(transpose(transpose(m)), m);
    let w = to_world(&i, [1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
    assert!(close(w, [5.0, 7.0, 9.0]));
}

/// 点积/叉积/归一化（退化返回零）/距离
#[test]
fn vec_math() {
    assert_eq!(dot([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]), 32.0);
    assert!(close(
        cross([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
        [0.0, 0.0, 1.0]
    ));
    assert!(close(normalize([0.0, 0.0, 2.0]), [0.0, 0.0, 1.0]));
    assert_eq!(normalize([0.0; 3]), [0.0; 3]);
    assert_eq!(dist([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]), 5.0);
}

/// 三角面法线 / Newell 环法线 / 环缩放
#[test]
fn face_and_ring_normal_scale() {
    let verts = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
    let n = face_normal(&verts, &[0, 1, 2]);
    assert!(close(n, [0.0, -1.0, 0.0]) || close(n, [0.0, 1.0, 0.0]));
    let square = [
        [1.0, 0.0, 1.0],
        [-1.0, 0.0, 1.0],
        [-1.0, 0.0, -1.0],
        [1.0, 0.0, -1.0],
    ];
    let rn = ring_normal(&square, &[0, 1, 2, 3]);
    assert!(close(rn, [0.0, 1.0, 0.0]) || close(rn, [0.0, -1.0, 0.0]));
    let mut ring = [[1.0, 2.0, 3.0]];
    scale_ring(&mut ring, [2.0, 1.0, 0.5]);
    assert!(close(ring[0], [2.0, 2.0, 1.5]));
}
