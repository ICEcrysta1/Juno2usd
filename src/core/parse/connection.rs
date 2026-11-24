//! Connection 解析：零件父子树（partA 是子，partB 是父）。

use roxmltree::Document;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Connection {
    /// 子零件 ID
    pub part_a: String,
    /// 父零件 ID
    pub part_b: String,
    #[allow(dead_code, reason = "连接点语义保留，骨骼模块落地后消费")]
    pub attach_a: String,
    #[allow(dead_code, reason = "连接点语义保留，骨骼模块落地后消费")]
    pub attach_b: String,
}

#[derive(Debug, Default)]
pub struct ConnectionTree {
    pub connections: Vec<Connection>,
    pub child_to_parent: HashMap<String, String>,
    pub parent_to_children: HashMap<String, Vec<String>>,
    /// 没有父零件的根零件
    pub root_parts: Vec<String>,
}

pub fn parse(doc: &Document) -> ConnectionTree {
    let mut tree = ConnectionTree::default();

    let Some(conns) = doc.descendants().find(|n| n.has_tag_name("Connections")) else {
        return tree;
    };

    for c in conns.children().filter(|n| n.has_tag_name("Connection")) {
        let (Some(part_a), Some(part_b)) = (c.attribute("partA"), c.attribute("partB")) else {
            continue;
        };
        let conn = Connection {
            part_a: part_a.to_string(),
            part_b: part_b.to_string(),
            attach_a: c.attribute("attachPointsA").unwrap_or("").to_string(),
            attach_b: c.attribute("attachPointsB").unwrap_or("").to_string(),
        };
        tree.child_to_parent
            .insert(conn.part_a.clone(), conn.part_b.clone());
        tree.parent_to_children
            .entry(conn.part_b.clone())
            .or_default()
            .push(conn.part_a.clone());
        tree.connections.push(conn);
    }

    let mut roots: Vec<String> = tree
        .parent_to_children
        .keys()
        .filter(|p| !tree.child_to_parent.contains_key(*p))
        .cloned()
        .collect();
    roots.sort();
    tree.root_parts = roots;
    tree
}

#[cfg(test)]
#[path = "../../../tests/src/core/parse/connection_test.rs"]
mod tests;
