//! UsdSkel 写入：SkelRoot + Skeleton（joints/bindTransforms/restTransforms）与网格绑定。

use crate::core::skel::Skeleton;
use openusd::gf::Mat4d;
use openusd::usd::Stage;
use openusd::vt::Value;
use openusd_schemas::geom::{Interpolation, PrimvarsAPI};
use openusd_schemas::skel::{BindingAPI, Root, Skeleton as UsdSkeleton, SkeletonSchema};

/// 在 parent 下创建 SkelRoot/Skeleton，返回 skeleton 路径
pub fn define(stage: &Stage, parent: &str, data: &Skeleton) -> Result<String, String> {
    let root_path = format!("{parent}/SkelRoot");
    Root::define(stage, &root_path).map_err(|e| e.to_string())?;
    let skel_path = format!("{root_path}/Skeleton");
    let skel = UsdSkeleton::define(stage, &skel_path).map_err(|e| e.to_string())?;

    skel.create_joints_attr()
        .map_err(|e| e.to_string())?
        .set(Value::TokenVec(
            data.joints.iter().map(|j| j.path.as_str().into()).collect(),
        ))
        .map_err(|e| e.to_string())?;
    skel.create_bind_transforms_attr()
        .map_err(|e| e.to_string())?
        .set(Value::Matrix4dVec(
            data.joints.iter().map(|j| Mat4d(j.bind)).collect(),
        ))
        .map_err(|e| e.to_string())?;
    skel.create_rest_transforms_attr()
        .map_err(|e| e.to_string())?
        .set(Value::Matrix4dVec(
            data.joints.iter().map(|j| Mat4d(j.rest)).collect(),
        ))
        .map_err(|e| e.to_string())?;

    Ok(skel_path)
}

/// 网格 100% 绑定到单个关节（vertex 插值 jointIndices/jointWeights）
pub fn bind(
    stage: &Stage,
    mesh_path: &str,
    skel_path: &str,
    joint_index: usize,
    vertex_count: usize,
) -> Result<(), String> {
    let prim = stage.prim(mesh_path).map_err(|e| e.to_string())?;
    BindingAPI::apply(&prim)
        .map_err(|e| e.to_string())?
        .create_skeleton_rel()
        .map_err(|e| e.to_string())?
        .add_target(skel_path)
        .map_err(|e| e.to_string())?;

    let pv = PrimvarsAPI::from_prim_unchecked(prim);
    pv.primvar_builder("skel:jointIndices", "int[]")
        .interpolation(Interpolation::Vertex)
        .set(Value::IntVec(vec![joint_index as i32; vertex_count]))
        .build()
        .map_err(|e| e.to_string())?;
    pv.primvar_builder("skel:jointWeights", "float[]")
        .interpolation(Interpolation::Vertex)
        .set(Value::FloatVec(vec![1.0; vertex_count]))
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}
