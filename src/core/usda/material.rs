//! 材质写入：UsdPreviewSurface（diffuseColor / metallic / roughness）与绑定。

use crate::core::parse::material::Material;
use openusd::gf::Vec3f;
use openusd::usd::Stage;
use openusd::vt::Value;
use openusd_schemas::shade::{Connectable, Material as UsdMaterial, MaterialBindingAPI, Shader};

/// 创建材质并绑定到网格；无材质数据时创建空材质（与旧导出器行为一致）
pub fn define_and_bind(
    stage: &Stage,
    root: &str,
    mat_name: &str,
    data: Option<&Material>,
    mesh_path: &str,
) -> Result<(), String> {
    let mat_path = format!("/{root}/Mat_{mat_name}");
    let material = UsdMaterial::define(stage, &mat_path).map_err(|e| e.to_string())?;

    if let Some(m) = data {
        let shader_path = format!("{mat_path}/PreviewSurface");
        let shader = Shader::define(stage, &shader_path).map_err(|e| e.to_string())?;
        shader
            .node_def()
            .create_id_attr()
            .map_err(|e| e.to_string())?
            .set(Value::token("UsdPreviewSurface"))
            .map_err(|e| e.to_string())?;
        shader
            .create_input("diffuseColor", "color3f")
            .map_err(|e| e.to_string())?
            .set(Value::Vec3f(Vec3f::from(m.color.map(|c| c as f32))))
            .map_err(|e| e.to_string())?;
        shader
            .create_input("metallic", "float")
            .map_err(|e| e.to_string())?
            .set(Value::Float(m.metallic as f32))
            .map_err(|e| e.to_string())?;
        shader
            .create_input("roughness", "float")
            .map_err(|e| e.to_string())?
            .set(Value::Float(m.roughness as f32))
            .map_err(|e| e.to_string())?;
        shader
            .create_output("surface", "token")
            .map_err(|e| e.to_string())?;
        material
            .create_surface_output()
            .map_err(|e| e.to_string())?
            .set_connections([format!("{shader_path}.outputs:surface")])
            .map_err(|e| e.to_string())?;
    }

    let prim = stage.prim(mesh_path).map_err(|e| e.to_string())?;
    MaterialBindingAPI::apply(&prim)
        .map_err(|e| e.to_string())?
        .bind(&mat_path)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/src/core/usda/material_test.rs"]
mod tests;
