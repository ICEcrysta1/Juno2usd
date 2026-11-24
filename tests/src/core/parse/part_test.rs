use super::*;
use crate::settings::Settings;

fn parse_one(xml: &str) -> Option<Part> {
    let doc = roxmltree::Document::parse(xml).unwrap();
    let elem = doc.descendants().find(|n| n.has_tag_name("Part")).unwrap();
    parse_part(elem, &[], &Settings::load().unwrap())
}

/// Fuselage 参数：length=2·offset_y、topScale、flattenNormals、Config/partScale
#[test]
fn fuselage_params_from_xml() {
    let p = parse_one(
        r#"<Part id="p" partType="Fuselage1" position="1,2,3" rotation="0,90,0" materials="0">
             <Fuselage offset="0,2,0" topScale="0,0" flattenNormals="Top"/>
             <Config partScale="2,1,1"/>
           </Part>"#,
    )
    .unwrap();
    assert_eq!(p.position, [1.0, 2.0, 3.0]);
    assert_eq!(p.rotation, [0.0, 90.0, 0.0]);
    assert_eq!(p.material_name, "default");
    let f = p.fuselage.unwrap();
    assert_eq!(f.length, 4.0);
    assert_eq!(f.top_scale, [0.0, 0.0]);
    assert!(f.flatten_normals.smooth_top());
    assert_eq!(f.part_scale, [2.0, 1.0, 1.0]);
}

/// 空心件壁厚来自 settings；Strut 缺省圆角
#[test]
fn inlet_wall_and_strut_corner_defaults() {
    let s = Settings::load().unwrap();
    let inlet =
        parse_one(r#"<Part id="i" partType="Inlet1"><Fuselage offset="0,1,0"/></Part>"#).unwrap();
    assert_eq!(inlet.wall_thickness, s.hollow.wall_thickness);
    let strut =
        parse_one(r#"<Part id="s" partType="Strut1"><Fuselage offset="0,1,0"/></Part>"#).unwrap();
    assert_eq!(
        strut.fuselage.unwrap().corner_radiuses,
        [s.fuselage.strut_corner_radius; 8]
    );
}

/// 未知 partType 跳过；向量/浮点列表解析容错
#[test]
fn unknown_type_and_number_parsing() {
    assert!(parse_one(r#"<Part id="x" partType="Foo1"/>"#).is_none());
    assert_eq!(parse_vec3("1,2,3"), Some([1.0, 2.0, 3.0]));
    assert_eq!(parse_vec3("1,2"), None);
    assert_eq!(parse_floats("1, x, 2.5"), vec![1.0, 2.5]);
}
