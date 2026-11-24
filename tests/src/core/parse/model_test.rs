use super::*;

/// partType 映射与关节判定
#[test]
fn part_type_from_xml_and_is_joint() {
    assert_eq!(PartType::from_xml("Fuselage1"), Some(PartType::Fuselage));
    assert_eq!(PartType::from_xml("Strut1"), Some(PartType::Strut));
    assert_eq!(PartType::from_xml("Inlet1"), Some(PartType::Inlet));
    assert_eq!(
        PartType::from_xml("FairingBase1"),
        Some(PartType::FairingBase)
    );
    assert_eq!(PartType::from_xml("Fairing1"), Some(PartType::Fairing));
    assert_eq!(PartType::from_xml("NoseCone1"), Some(PartType::NoseCone));
    assert_eq!(
        PartType::from_xml("FairingNoseCone1"),
        Some(PartType::FairingNoseCone)
    );
    assert_eq!(PartType::from_xml("Rotator1"), Some(PartType::Rotator));
    assert_eq!(
        PartType::from_xml("HingeRotator1"),
        Some(PartType::HingeRotator)
    );
    assert_eq!(PartType::from_xml("Unknown"), None);
    assert!(PartType::Rotator.is_joint());
    assert!(PartType::HingeRotator.is_joint());
    assert!(!PartType::Fuselage.is_joint());
}

/// flattenNormals 游戏语义：被覆盖的端 = 开平滑
#[test]
fn flatten_normals_switch_semantics() {
    assert_eq!(FlattenNormals::from_xml("Top"), FlattenNormals::Top);
    assert_eq!(FlattenNormals::from_xml("Bottom"), FlattenNormals::Bottom);
    assert_eq!(FlattenNormals::from_xml("Both"), FlattenNormals::Both);
    assert_eq!(FlattenNormals::from_xml("junk"), FlattenNormals::None);
    assert!(!FlattenNormals::None.smooth_top());
    assert!(!FlattenNormals::None.smooth_bottom());
    assert!(FlattenNormals::Top.smooth_top());
    assert!(!FlattenNormals::Top.smooth_bottom());
    assert!(FlattenNormals::Bottom.smooth_bottom());
    assert!(FlattenNormals::Both.smooth_top() && FlattenNormals::Both.smooth_bottom());
}
