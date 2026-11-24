use super::*;

/// 全链路：parse→几何→法线→混合→骨骼→dump/usda，Report 统计正确
#[test]
fn full_pipeline_report_and_artifacts() {
    let dir = std::env::temp_dir().join(format!("juno2usd-convert-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("ship.xml");
    std::fs::write(
        &input,
        r#"<Ship>
             <DesignerSettings><Theme>
               <Material name="Mat A" color="FF0000" m="0.25" s="0.5"/>
             </Theme></DesignerSettings>
             <Part id="body" partType="Fuselage1" position="0,0,0" rotation="0,0,0" materials="0">
               <Fuselage offset="0,1,0"/>
             </Part>
             <Part id="rot" partType="Rotator1" position="0,2,0" rotation="0,0,0"/>
             <Connections><Connection partA="body" partB="rot"/></Connections>
           </Ship>"#,
    )
    .unwrap();
    let dump = dir.join("dump.json");
    let out = dir.join("ship.usda");
    let mut msgs = Vec::new();
    let r = convert(&input, Some(&dump), Some(&out), |m| {
        msgs.push(m.to_string())
    })
    .unwrap();
    assert_eq!(r.parts, 2);
    assert_eq!(r.materials, 1);
    assert_eq!(r.connections, 1);
    assert_eq!(r.joints, 1);
    assert_eq!(r.bindings, 2);
    // 24 段圆柱：2*24+2 顶点、4*24 面
    assert_eq!(r.vertices, 50);
    assert_eq!(r.faces, 96);
    assert!(msgs.iter().any(|m| m.contains("几何+法线 2/2")));
    let dj = std::fs::read_to_string(&dump).unwrap();
    assert!(dj.contains("\"body\"") && dj.contains("\"vertices\""));
    let usda = std::fs::read_to_string(&out).unwrap();
    assert!(usda.contains("SkelRoot") && usda.contains("UsdPreviewSurface"));
    std::fs::remove_dir_all(&dir).unwrap();
}
