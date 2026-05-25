#!/usr/bin/env python3
"""USD 合并导出模块。接收 MeshData 列表，写入 .usda 文件。"""

from __future__ import annotations

import os
from typing import Dict, List, Optional, Tuple

from .models import Material, MeshData


class MeshMerger:
    """网格合并器：将多个 MeshData 合并写入单个 USDA 文件。"""

    def __init__(self) -> None:
        self._meshes: List[MeshData] = []
        self._materials: Dict[str, Material] = {}

    def set_meshes(self, meshes: List[MeshData]) -> None:
        """设置要导出的网格列表。

        Args:
            meshes: MeshData 列表（含法线）。
        """
        self._meshes = meshes
        print(f"[Merger] 已加载 {len(meshes)} 个网格")

    def set_materials(self, materials: List[Material]) -> None:
        """设置材质列表。

        Args:
            materials: Material 对象列表。
        """
        self._materials = {m.name: m for m in materials}
        print(f"[Merger] 已加载 {len(materials)} 种材质")

    def merge_and_export(
        self,
        output_file: str,
        mesh_prefix: str = "Mesh",
        use_custom_normals: bool = True,
    ) -> None:
        """合并网格并导出 USDA 文件。

        Args:
            output_file: 输出 .usda 文件路径。
            mesh_prefix: 网格名称前缀。
            use_custom_normals: 是否使用自定义法线。
        """
        try:
            from pxr import Gf, Sdf, Usd, UsdGeom, UsdShade
        except ImportError:
            print("[Merger] 错误: 未安装 pxr (USD) 库，请运行 run_pipeline.bat 自动安装")
            return

        output_dir = os.path.dirname(output_file)
        if output_dir:
            os.makedirs(output_dir, exist_ok=True)

        stage = Usd.Stage.CreateNew(output_file)
        root_path = Sdf.Path(f"/{mesh_prefix}")
        UsdGeom.Xform.Define(stage, root_path)

        # 按材质分组
        material_groups: Dict[str, List[MeshData]] = {}
        for mesh in self._meshes:
            mat_name = mesh.faces[0][9] if mesh.faces else "default"
            material_groups.setdefault(mat_name, []).append(mesh)

        mesh_index = 0
        for mat_name, group_meshes in material_groups.items():
            for mesh in group_meshes:
                mesh_name = f"Mesh_{mesh_index}"
                mesh_path = root_path.AppendChild(mesh_name)
                mesh_prim = UsdGeom.Mesh.Define(stage, mesh_path)

                vertices = mesh.vertices
                normals = mesh.normals
                uvs = mesh.uvs
                faces = mesh.faces

                # 过滤退化面
                valid_faces = []
                for face in faces:
                    *face_data, mat, pid = face
                    v1, v2, v3 = face_data[0], face_data[1], face_data[2]
                    if v1 != v2 and v2 != v3 and v3 != v1:
                        valid_faces.append(face)
                faces = valid_faces

                if not faces:
                    continue

                # 本地索引重映射
                global_to_local: Dict[Tuple[int, int], Tuple[int, List[Tuple]]] = {}
                local_vertices: List[Tuple[float, float, float]] = []
                local_faces: List[List[int]] = []

                for face in faces:
                    local_face: List[int] = []
                    for corner in range(3):
                        v_idx = face[corner]
                        vt_idx = face[corner + 3]
                        key = (v_idx, vt_idx)

                        if key not in global_to_local:
                            li = len(local_vertices)
                            global_to_local[key] = (li, [])
                            local_vertices.append(vertices[v_idx])
                        local_face.append(global_to_local[key][0])
                    local_faces.append(local_face)

                # 设置顶点
                mesh_prim.CreatePointsAttr([Gf.Vec3f(*v) for v in local_vertices])

                # 设置法线
                if use_custom_normals and normals:
                    face_varying_normals: List[Gf.Vec3f] = []
                    for face in faces:
                        for corner in range(3):
                            v_idx = face[corner]
                            vt_idx = face[corner + 3]
                            ni = face[corner + 6]
                            if 0 <= ni < len(normals):
                                face_varying_normals.append(Gf.Vec3f(*normals[ni]))
                            else:
                                face_varying_normals.append(Gf.Vec3f(0.0, 1.0, 0.0))
                    mesh_prim.CreateNormalsAttr(face_varying_normals)
                    mesh_prim.SetNormalsInterpolation(UsdGeom.Tokens.faceVarying)

                # 设置 UV
                if uvs:
                    local_uvs: List[Tuple[float, float]] = []
                    uv_map: Dict[int, Tuple[float, float]] = {}
                    for face in faces:
                        for corner in range(3):
                            v_idx = face[corner]
                            vt_idx = face[corner + 3]
                            key = (v_idx, vt_idx)
                            if key in global_to_local:
                                local_idx = global_to_local[key][0]
                                if local_idx not in uv_map:
                                    if vt_idx < len(uvs):
                                        uv_map[local_idx] = uvs[vt_idx]
                                    else:
                                        uv_map[local_idx] = (0.0, 0.0)
                    local_uvs = [uv_map.get(i, (0.0, 0.0)) for i in range(len(local_vertices))]
                    tex_coords = UsdGeom.PrimvarsAPI(mesh_prim).CreatePrimvar(
                        "st", Sdf.ValueTypeNames.TexCoord2fArray, UsdGeom.Tokens.vertex,
                    )
                    tex_coords.Set(local_uvs)

                # 设置面数
                mesh_prim.CreateFaceVertexCountsAttr([3] * len(local_faces))
                mesh_prim.CreateFaceVertexIndicesAttr(
                    [idx for face in local_faces for idx in face]
                )

                # 创建材质
                if mat_name and mat_name in self._materials:
                    mat_path = root_path.AppendChild(f"Mat_{mesh_name}")
                    material = UsdShade.Material.Define(stage, mat_path)

                    mat_data = self._materials[mat_name]
                    shader_path = mat_path.AppendChild("PreviewSurface")
                    shader = UsdShade.Shader.Define(stage, shader_path)
                    shader.CreateIdAttr("UsdPreviewSurface")

                    diffuse = Gf.Vec3f(*mat_data.color)
                    shader.CreateInput("diffuseColor", Sdf.ValueTypeNames.Color3f).Set(diffuse)
                    shader.CreateInput("metallic", Sdf.ValueTypeNames.Float).Set(mat_data.metallic)
                    shader.CreateInput("roughness", Sdf.ValueTypeNames.Float).Set(mat_data.roughness)

                    shader_output = shader.CreateOutput("surface", Sdf.ValueTypeNames.Token)
                    material.CreateSurfaceOutput().ConnectToSource(shader_output)
                    UsdShade.MaterialBindingAPI(mesh_prim).Bind(material)

                mesh_index += 1

        stage.GetRootLayer().Save()
        print(f"[Merger] USD文件已保存: {output_file}")
