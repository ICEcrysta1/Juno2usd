//! 材质定义与 Theme 解析。
//!
//! hex 颜色 -> RGB，m = metallic，roughness = 1 - s（游戏光滑度反转）。

use roxmltree::Document;

#[derive(Debug, Clone)]
pub struct Material {
    pub name: String,
    /// RGB 0-1
    pub color: [f64; 3],
    pub metallic: f64,
    pub roughness: f64,
}

/// 清理名称以符合 USD 命名规范
pub fn clean_name(name: &str) -> String {
    name.replace([' ', '-', '.', '/'], "_")
}

/// 从 DesignerSettings/Theme 或 Themes/Theme 解析材质列表
pub fn parse(doc: &Document) -> Vec<Material> {
    let theme = doc
        .descendants()
        .find(|n| {
            n.has_tag_name("Theme")
                && n.parent()
                    .is_some_and(|p| p.has_tag_name("DesignerSettings"))
        })
        .or_else(|| {
            doc.descendants().find(|n| {
                n.has_tag_name("Theme") && n.parent().is_some_and(|p| p.has_tag_name("Themes"))
            })
        });

    let Some(theme) = theme else {
        return Vec::new();
    };

    theme
        .children()
        .filter(|n| n.has_tag_name("Material"))
        .map(|m| {
            let name = m.attribute("name").unwrap_or("Unknown").to_string();
            let color = parse_hex_color(m.attribute("color").unwrap_or("FFFFFF"));
            let metallic = m.attribute("m").and_then(|v| v.parse().ok()).unwrap_or(0.0);
            let smoothness = m.attribute("s").and_then(|v| v.parse().ok()).unwrap_or(0.0);
            Material {
                name,
                color,
                metallic,
                roughness: 1.0 - smoothness,
            }
        })
        .collect()
}

fn parse_hex_color(hex: &str) -> [f64; 3] {
    let channel = |range: std::ops::Range<usize>| {
        u8::from_str_radix(hex.get(range).unwrap_or(""), 16)
            .map(|v| v as f64 / 255.0)
            .unwrap_or(1.0)
    };
    if hex.len() >= 6 {
        [channel(0..2), channel(2..4), channel(4..6)]
    } else {
        [1.0, 1.0, 1.0]
    }
}

#[cfg(test)]
#[path = "../../../tests/src/core/parse/material_test.rs"]
mod tests;
