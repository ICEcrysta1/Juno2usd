//! 舞台与根结构：metersPerUnit = 1，upAxis = Y，根 Xform 为 defaultPrim。

use openusd::usd::Stage;
use openusd::vt::Value;

/// 创建内存舞台与根 prim，返回 (舞台, 根名称)
pub fn create(prefix: &str) -> Result<(Stage, String), String> {
    let stage = Stage::builder()
        .schema_registry(openusd_schemas::schema_registry())
        .in_memory("juno2usd_export.usda")
        .map_err(|e| e.to_string())?;

    stage
        .set_stage_metadata("metersPerUnit", Value::Double(1.0))
        .map_err(|e| e.to_string())?;
    stage
        .set_stage_metadata("upAxis", Value::Token("Y".into()))
        .map_err(|e| e.to_string())?;

    let mut root = crate::core::parse::material::clean_name(prefix);
    if root.is_empty() {
        root = "Model".to_string();
    }
    if root.starts_with(|c: char| c.is_ascii_digit()) {
        root = format!("_{root}");
    }

    stage
        .define_typed_prim(format!("/{root}"), "Xform")
        .map_err(|e| e.to_string())?;
    stage
        .set_default_prim(root.clone())
        .map_err(|e| e.to_string())?;

    Ok((stage, root))
}
