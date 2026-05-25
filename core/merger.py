#!/usr/bin/env python3
"""USD 合并导出模块。接收 MeshData 列表，写入 .usda 文件。"""

from __future__ import annotations

import os
from typing import Dict, List, Optional, Tuple

from .models import Material, MeshData
from .utils.naming import clean_material_name


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
        self._materials = {clean_material_name(m.name): m for m in materials}
        print(f"[Merger] 已加载 {len(materials)} 种材质")

    def merge_and_export(
        self,
        output_file: str,
        mesh_prefix: str = "Mesh",
        use_custom_normals: bool = True,
    ) -> None:
        """合并网格并导出 USDA 文件。相同材质的零件合并为一个 Mesh。

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
            if mat_name not in material_groups:
                material_groups[mat_name] = []
            material_groups[mat_name].append(mesh)

        print(f"[Merger] 材质分组: {len(material_groups)} 组")

        for mat_name, group_meshes in material_groups.items():
            clean_name = clean_material_name(mat_name) if mat_name else "default"
            mesh_name = f"Mesh_{clean_name}"
            mesh_path = root_path.AppendChild(mesh_name)
            mesh_prim = UsdGeom.Mesh.Define(stage, mesh_path)

            # 合并同材质所有零件的顶点/法线/UV/面
            all_vertices: List[Tuple[float, float, float]] = []
            all_normals: List[Tuple[float, float, float]] = []
            all_uvs: List[Tuple[float, float]] = []
            all_faces_normal_indices: List[Tuple[int, int, int]] = []
            all_face_vertex_indices: List[Tuple[int, int, int]] = []

            for mesh in group_meshes:
                # 过滤退化面
                valid_faces = []
                for face in mesh.faces:
                    v1, v2, v3 = face[0], face[1], face[2]
                    if v1 != v2 and v2 != v3 and v3 != v1:
                        valid_faces.append(face)

                if not valid_faces:
                    continue

                # 以 (v_idx, uv_idx) 为键展开顶点，确保位置和 UV 数组 1:1 对齐
                # 法线使用 faceVarying 插值，独立存储
                local_to_global_v: Dict[Tuple[int, int], int] = {}
                local_to_global_n: Dict[int, int] = {}

                for face in valid_faces:
                    for corner in range(3):
                        vi = face[corner]
                        ni = face[corner + 6]
                        uvi = face[corner + 3]
                        key = (vi, uvi)

                        if key not in local_to_global_v:
                            local_to_global_v[key] = len(all_vertices)
                            all_vertices.append(mesh.vertices[vi])
                            if 0 <= uvi < len(mesh.uvs):
                                all_uvs.append(mesh.uvs[uvi])
                            else:
                                all_uvs.append((0.0, 0.0))

                        if 0 <= ni < len(mesh.normals) and ni not in local_to_global_n:
                            local_to_global_n[ni] = len(all_normals)
                            if use_custom_normals:
                                all_normals.append(mesh.normals[ni])
                            else:
                                all_normals.append((0.0, 1.0, 0.0))

                # 输出面索引
                for face in valid_faces:
                    fvi = tuple(
                        local_to_global_v[(face[corner], face[corner + 3])]
                        for corner in range(3)
                    )
                    fni = tuple(
                        local_to_global_n.get(face[corner + 6], 0)
                        for corner in range(3)
                    )
                    all_face_vertex_indices.append(fvi)
                    all_faces_normal_indices.append(fni)

            if not all_face_vertex_indices:
                continue

            # 设置顶点
            mesh_prim.CreatePointsAttr([Gf.Vec3f(*v) for v in all_vertices])

            # 设置法线
            if use_custom_normals and all_normals:
                face_varying_normals: List[Gf.Vec3f] = []
                for fni in all_faces_normal_indices:
                    for ni in fni:
                        if 0 <= ni < len(all_normals):
                            face_varying_normals.append(Gf.Vec3f(*all_normals[ni]))
                        else:
                            face_varying_normals.append(Gf.Vec3f(0.0, 1.0, 0.0))
                mesh_prim.CreateNormalsAttr(face_varying_normals)
                mesh_prim.SetNormalsInterpolation(UsdGeom.Tokens.faceVarying)

            # 设置 UV
            if all_uvs:
                tex_coords = UsdGeom.PrimvarsAPI(mesh_prim).CreatePrimvar(
                    "st", Sdf.ValueTypeNames.TexCoord2fArray, UsdGeom.Tokens.vertex,
                )
                tex_coords.Set([(uv[0], uv[1]) for uv in all_uvs])

            # 设置面
            mesh_prim.CreateFaceVertexCountsAttr([3] * len(all_face_vertex_indices))
            mesh_prim.CreateFaceVertexIndicesAttr(
                [idx for fvi in all_face_vertex_indices for idx in fvi]
            )

            # 创建材质
            lookup_key = clean_material_name(mat_name) if mat_name else "default"
            if lookup_key in self._materials:
                mat_path = root_path.AppendChild(f"Mat_{clean_name}")
                material = UsdShade.Material.Define(stage, mat_path)
                mat_data = self._materials[lookup_key]
                shader_path = mat_path.AppendChild("PreviewSurface")
                shader = UsdShade.Shader.Define(stage, shader_path)
                shader.CreateIdAttr("UsdPreviewSurface")
                shader.CreateInput("diffuseColor", Sdf.ValueTypeNames.Color3f).Set(Gf.Vec3f(*mat_data.color))
                shader.CreateInput("metallic", Sdf.ValueTypeNames.Float).Set(mat_data.metallic)
                shader.CreateInput("roughness", Sdf.ValueTypeNames.Float).Set(mat_data.roughness)
                shader_output = shader.CreateOutput("surface", Sdf.ValueTypeNames.Token)
                material.CreateSurfaceOutput().ConnectToSource(shader_output)
                UsdShade.MaterialBindingAPI(mesh_prim).Bind(material)

        stage.GetRootLayer().Save()
        print(f"[Merger] USD文件已保存: {output_file}")
