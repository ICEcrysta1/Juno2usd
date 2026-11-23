//! 变换数学：Unity 欧拉角旋转与局部→世界变换。
//!
//! 约定：几何一律在局部空间生成，最后经此变换到世界空间。

pub type Vec3 = [f64; 3];

/// Unity 欧拉角（度数，ZXY 顺序）-> 旋转矩阵 R = Ry·Rx·Rz
pub fn rotation_matrix(euler_deg: Vec3) -> [[f64; 3]; 3] {
    let [rx, ry, rz] = euler_deg.map(f64::to_radians);
    let (cz, sz) = (rz.cos(), rz.sin());
    let (cx, sx) = (rx.cos(), rx.sin());
    let (cy, sy) = (ry.cos(), ry.sin());

    let rz_m = [[cz, -sz, 0.0], [sz, cz, 0.0], [0.0, 0.0, 1.0]];
    let rx_m = [[1.0, 0.0, 0.0], [0.0, cx, -sx], [0.0, sx, cx]];
    let ry_m = [[cy, 0.0, sy], [0.0, 1.0, 0.0], [-sy, 0.0, cy]];

    mat_mul(ry_m, mat_mul(rx_m, rz_m))
}

fn mat_mul(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut out = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            out[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    out
}

/// 世界坐标 = R·局部坐标 + position
pub fn to_world(rot: &[[f64; 3]; 3], pos: Vec3, p: Vec3) -> Vec3 {
    let r = rotate_vec(rot, p);
    [r[0] + pos[0], r[1] + pos[1], r[2] + pos[2]]
}

/// 纯旋转向量（法线变换用，无平移）
pub fn rotate_vec(rot: &[[f64; 3]; 3], v: Vec3) -> Vec3 {
    [
        rot[0][0] * v[0] + rot[0][1] * v[1] + rot[0][2] * v[2],
        rot[1][0] * v[0] + rot[1][1] * v[1] + rot[1][2] * v[2],
        rot[2][0] * v[0] + rot[2][1] * v[1] + rot[2][2] * v[2],
    ]
}

pub fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// 归一化；退化向量返回零向量
pub fn normalize(v: Vec3) -> Vec3 {
    let len = dot(v, v).sqrt();
    if len > 1e-12 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        [0.0; 3]
    }
}

/// 环坐标缩放（partScale）
pub fn scale_ring(ring: &mut [Vec3], scale: Vec3) {
    for c in ring.iter_mut() {
        c[0] *= scale[0];
        c[1] *= scale[1];
        c[2] *= scale[2];
    }
}
