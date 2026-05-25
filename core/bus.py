#!/usr/bin/env python3
"""总线：遍历零件 → 路由到零件模块 → 调共享服务 → 组装 MeshData。"""

from __future__ import annotations

from typing import Dict, List, Optional, Tuple, Type

from .models import FuselageParams, MeshData, PartData, RawGeometry
from .parts.base import PartBase
from .parts.fuselage import FuselagePart
from .parts.strut import StrutPart
from .parts.inlet import InletPart
from .parts.fairing_base import FairingBasePart
from .parts.fairing import FairingPart
from .parts.nose_cone import NoseConePart
from .parts.fairing_nose_cone import FairingNoseConePart
from .shared.transform import TransformService
from .shared.normal import NormalService
from .shared.material import MaterialService


# 零件类型 → 零件模块路由表
PART_REGISTRY: Dict[str, Type[PartBase]] = {
    "Fuselage1": FuselagePart,
    "Strut1": StrutPart,
    "Inlet1": InletPart,
    "FairingBase1": FairingBasePart,
    "Fairing1": FairingPart,
    "NoseCone1": NoseConePart,
    "FairingNoseCone1": FairingNoseConePart,
}

# 不需要生成几何的关节类型
JOINT_TYPES = frozenset({"Rotator1", "HingeRotator1"})


class Bus:
    """流水线总线：调度零件生成几何并组装完整网格。"""

    def __init__(self, segments: int = 24) -> None:
        self._segments = segments
        self._transform = TransformService()
        self._normal = NormalService()
        self._material = MaterialService()
        self._part_instances: Dict[str, PartBase] = {}

    def assemble(
        self,
        parts: List[PartData],
        connections: Optional[Dict[str, Optional[str]]] = None,
    ) -> List[MeshData]:
        """组装所有零件为带法线的完整网格列表。

        Args:
            parts: PartData 列表。
            connections: child_to_parent 映射（可选的骨骼预备数据）。

        Returns:
            MeshData 列表。
        """
        if connections is None:
            connections = {}

        meshes: List[MeshData] = []
        print(f"[Bus] 开始组装，共 {len(parts)} 个零件")

        for part in parts:
            # 跳过关节零件
            if part.part_type in JOINT_TYPES:
                continue

            # 跳过无 Fuselage 参数的零件
            if not part.fuselage_params:
                continue

            part_cls = PART_REGISTRY.get(part.part_type)
            if part_cls is None:
                print(f"[Bus] 跳过不支持的零件类型: {part.part_type}")
                continue

            # 1. 获取或创建零件模块实例
            part_instance = self._get_part_instance(part.part_type, part_cls)

            # 2. 零件生成裸几何
            geom = part_instance.generate(part.fuselage_params, self._segments)
            geom.part_id = part.part_id
            geom.material_name = part.material_name

            # 更新 face 中的 material_name（generate 时用的空字符串）
            updated_faces = []
            for face in geom.faces:
                face_list = list(face)
                face_list[7] = part.material_name  # 材质在 8 值面元组中索引为 7
                updated_faces.append(tuple(face_list))
            geom.faces = updated_faces

            print(f"[Bus]   {part.part_id} ({part.part_type}): "
                  f"{len(geom.vertices)} 顶点, {len(geom.faces)} 面")

            # 3. 应用变换
            self._transform.apply_rotation(geom, part.rotation)
            self._transform.apply_position(geom, part.position)

            # 4. 计算法线
            normals = self._normal.compute(geom)

            # 5. 查找父级（骨骼预留）
            parent = part_instance.find_parent(part.part_id, connections)
            geom.parent_part_id = parent

            # 6. 组装为 MeshData
            mesh = self._to_meshdata(geom, normals, part)
            meshes.append(mesh)

        print(f"[Bus] 组装完成，共 {len(meshes)} 个网格")
        return meshes

    def _get_part_instance(self, part_type: str, part_cls: Type[PartBase]) -> PartBase:
        """获取或缓存零件模块实例。"""
        if part_type not in self._part_instances:
            self._part_instances[part_type] = part_cls()
        return self._part_instances[part_type]

    @staticmethod
    def _to_meshdata(
        geom: RawGeometry, normals: List[Tuple[float, float, float]], part: PartData
    ) -> MeshData:
        """将 RawGeometry + normals 转为 MeshData。"""
        mesh = MeshData(part_id=geom.part_id)
        mesh.vertices = geom.vertices
        mesh.uvs = geom.uvs
        mesh.normals = normals

        # NormalService.compute() 已将面格式从 8 值转为 11 值
        # (v1,v2,v3,vt1,vt2,vt3,vn1,vn2,vn3,mat,pid)
        for face in geom.faces:
            if len(face) == 11:
                mesh.faces.append(face)
            elif len(face) == 8:
                v1, v2, v3, vt1, vt2, vt3, mat, pid = face
                mesh.faces.append((v1, v2, v3, vt1, vt2, vt3, 0, 0, 0, mat, pid))
            else:
                # 降级：取前 11 个
                mesh.faces.append(tuple(face[:11]))

        return mesh
