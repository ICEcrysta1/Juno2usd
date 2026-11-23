//! 网格写入：points / faceVertexCounts / faceVertexIndices / st primvar。
//!
//! 法线待 normal 模块落地后以 faceVarying 写入；当前 subdivisionScheme = none，
//! 查看器按面片渲染。

use openusd::gf::{Vec2f, Vec3f};
use openusd::usd::Stage;
use openusd::vt::Value;
use openusd_schemas::geom::{Interpolation, Mesh, MeshSchema, PointBasedSchema, PrimvarsAPI};

/// 紧凑化后的网格数据（USD 精度为 f32）
pub struct CompactMesh {
    pub points: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub faces: Vec<[i32; 3]>,
}

pub fn write(stage: &Stage, path: &str, data: &CompactMesh) -> Result<(), String> {
    let mesh = Mesh::define(stage, path).map_err(|e| e.to_string())?;

    mesh.create_points_attr()
        .map_err(|e| e.to_string())?
        .set(Value::Vec3fVec(
            data.points.iter().map(|&p| Vec3f::from(p)).collect(),
        ))
        .map_err(|e| e.to_string())?;
    mesh.create_face_vertex_counts_attr()
        .map_err(|e| e.to_string())?
        .set(Value::IntVec(vec![3; data.faces.len()]))
        .map_err(|e| e.to_string())?;
    mesh.create_face_vertex_indices_attr()
        .map_err(|e| e.to_string())?
        .set(Value::IntVec(
            data.faces.iter().flatten().copied().collect(),
        ))
        .map_err(|e| e.to_string())?;
    mesh.create_subdivision_scheme_attr()
        .map_err(|e| e.to_string())?
        .set(Value::token("none"))
        .map_err(|e| e.to_string())?;

    // UV：st primvar，vertex 插值（UV 与顶点一一对应）
    let primvars = PrimvarsAPI::from_prim_unchecked(stage.prim(path).map_err(|e| e.to_string())?);
    primvars
        .primvar_builder("st", "texCoord2f[]")
        .interpolation(Interpolation::Vertex)
        .set(Value::Vec2fVec(
            data.uvs.iter().map(|&uv| Vec2f::from(uv)).collect(),
        ))
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}
