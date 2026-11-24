//! 零件解析：partType 分发、Fuselage 参数、Config/partScale。

use super::model::{FlattenNormals, FuselageParams, ParsedModel, Part, PartType};
use super::{connection, material};
use crate::core::shapes::algo::transform::Vec3;
use crate::settings::Settings;
use roxmltree::Node;
use std::path::Path;

/// 解析 SR2 XML：材质 + 零件 + 连接
pub fn parse_file(path: &Path, settings: &Settings) -> Result<ParsedModel, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("读取失败: {e}"))?;
    let doc = roxmltree::Document::parse(&text).map_err(|e| format!("XML 解析失败: {e}"))?;

    let materials = material::parse(&doc);
    let parts = doc
        .descendants()
        .filter(|n| n.has_tag_name("Part"))
        .filter_map(|elem| parse_part(elem, &materials, settings))
        .collect();
    let connections = connection::parse(&doc);

    Ok(ParsedModel {
        materials,
        parts,
        connections,
    })
}

fn parse_part(elem: Node, materials: &[material::Material], settings: &Settings) -> Option<Part> {
    let part_type = PartType::from_xml(elem.attribute("partType").unwrap_or("unknown"))?;
    let id = elem.attribute("id").unwrap_or("unknown").to_string();
    let position = elem
        .attribute("position")
        .and_then(parse_vec3)
        .unwrap_or([0.0; 3]);
    let rotation = elem
        .attribute("rotation")
        .and_then(parse_vec3)
        .unwrap_or([0.0; 3]);

    let material_index = elem
        .attribute("materials")
        .unwrap_or("0,0,0,0,0")
        .split(',')
        .next()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);
    let material_name = materials
        .get(material_index)
        .map(|m| material::clean_name(&m.name))
        .unwrap_or_else(|| "default".to_string());

    let mut part = Part {
        id,
        part_type,
        position,
        rotation,
        material_index,
        material_name,
        fuselage: None,
        wall_thickness: 0.0,
        subdivisions: settings.mesh.nose_cone_subdivisions,
    };

    match part_type {
        PartType::Fuselage | PartType::Strut | PartType::NoseCone => {
            part.fuselage = parse_fuselage_params(elem, settings, part_type);
        }
        PartType::Inlet | PartType::FairingBase | PartType::Fairing => {
            part.fuselage = parse_fuselage_params(elem, settings, part_type);
            part.wall_thickness = settings.hollow.wall_thickness;
        }
        PartType::FairingNoseCone => {
            part.fuselage = parse_fuselage_params(elem, settings, part_type);
            part.wall_thickness = settings.hollow.wall_thickness;
        }
        PartType::Rotator | PartType::HingeRotator => {}
    }

    Some(part)
}

/// 解析 <Fuselage> 子元素与 Config/partScale
fn parse_fuselage_params(
    elem: Node,
    settings: &Settings,
    part_type: PartType,
) -> Option<FuselageParams> {
    let fus = elem.children().find(|n| n.has_tag_name("Fuselage"))?;

    let mut p = FuselageParams {
        length: 5.0,
        radius_x: settings.fuselage.default_radius_x,
        radius_z: settings.fuselage.default_radius_z,
        offset: [0.0; 3],
        top_scale: [1.0; 2],
        bottom_scale: [1.0; 2],
        deformations: [0.0; 3],
        corner_radiuses: [1.0; 8],
        clamp_distances: [1.0; 8],
        part_scale: [1.0; 3],
        flatten_normals: FlattenNormals::None,
    };

    if let Some(offset) = fus.attribute("offset").and_then(parse_vec3) {
        p.offset = offset;
    }
    if let Some(s) = fus.attribute("topScale").map(parse_floats)
        && s.len() >= 2
    {
        p.top_scale = [s[0], s[1]];
    }
    if let Some(s) = fus.attribute("bottomScale").map(parse_floats)
        && s.len() >= 2
    {
        p.bottom_scale = [s[0], s[1]];
    }
    if let Some(d) = fus.attribute("deformations").and_then(parse_vec3) {
        p.deformations = d;
    }
    if let Some(r) = fus.attribute("cornerRadiuses").map(parse_floats)
        && r.len() >= 8
    {
        p.corner_radiuses.copy_from_slice(&r[..8]);
    }
    if let Some(c) = fus.attribute("clampDistances").map(parse_floats)
        && c.len() >= 8
    {
        p.clamp_distances.copy_from_slice(&c[..8]);
    }

    p.length = p.offset[1] * 2.0;

    if let Some(f) = fus.attribute("flattenNormals") {
        p.flatten_normals = FlattenNormals::from_xml(f);
    }

    if let Some(config) = elem.children().find(|n| n.has_tag_name("Config"))
        && let Some(s) = config.attribute("partScale").and_then(parse_vec3)
    {
        p.part_scale = s;
    }

    // Strut1 缺省圆角
    if part_type == PartType::Strut && fus.attribute("cornerRadiuses").is_none() {
        p.corner_radiuses = [settings.fuselage.strut_corner_radius; 8];
    }

    Some(p)
}

/// 解析逗号分隔的三维向量
fn parse_vec3(text: &str) -> Option<Vec3> {
    let v = parse_floats(text);
    if v.len() == 3 {
        Some([v[0], v[1], v[2]])
    } else {
        None
    }
}

/// 解析逗号分隔的浮点列表（跳过非法项）
fn parse_floats(text: &str) -> Vec<f64> {
    text.split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect()
}

#[cfg(test)]
#[path = "../../../tests/src/core/parse/part_test.rs"]
mod tests;
