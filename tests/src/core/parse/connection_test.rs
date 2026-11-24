use super::*;

/// 父子树：partA 子 / partB 父，根排序
#[test]
fn tree_parent_child_and_roots() {
    let doc = roxmltree::Document::parse(
        r#"<Ship><Connections>
             <Connection partA="b" partB="a"/>
             <Connection partA="c" partB="a"/>
             <Connection partA="a" partB="r"/>
           </Connections></Ship>"#,
    )
    .unwrap();
    let t = parse(&doc);
    assert_eq!(t.connections.len(), 3);
    assert_eq!(t.child_to_parent["b"], "a");
    assert_eq!(t.parent_to_children["a"], vec!["b", "c"]);
    assert_eq!(t.root_parts, vec!["r"]);
}

/// 缺 Connections / 缺属性 → 空树/跳过
#[test]
fn missing_connections_default() {
    let doc = roxmltree::Document::parse("<Ship/>").unwrap();
    let t = parse(&doc);
    assert!(t.connections.is_empty() && t.root_parts.is_empty());
    let doc = roxmltree::Document::parse(r#"<Connections><Connection partA="x"/></Connections>"#)
        .unwrap();
    assert!(parse(&doc).connections.is_empty());
}
