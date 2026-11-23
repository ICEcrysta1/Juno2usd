//! Part 公共数学：Unity 欧拉角旋转与局部→世界变换。
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
    [
        rot[0][0] * p[0] + rot[0][1] * p[1] + rot[0][2] * p[2] + pos[0],
        rot[1][0] * p[0] + rot[1][1] * p[1] + rot[1][2] * p[2] + pos[1],
        rot[2][0] * p[0] + rot[2][1] * p[1] + rot[2][2] * p[2] + pos[2],
    ]
}
