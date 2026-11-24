use super::*;

/// 内嵌权威默认参数可加载且值正确
#[test]
fn load_embedded_defaults() {
    let s = Settings::load().unwrap();
    assert_eq!(s.mesh.segments, 24);
    assert_eq!(s.mesh.nose_cone_subdivisions, 5);
    assert_eq!(s.fuselage.default_radius_x, 1.0);
    assert_eq!(s.fuselage.strut_corner_radius, 0.5);
    assert_eq!(s.hollow.wall_thickness, 0.045);
    assert_eq!(s.normals.sharp_edge_angle, 60.0);
}
