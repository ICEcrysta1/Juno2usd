#!/usr/bin/env python3
"""纯数据模型。零件模块、共享服务和流水线之间的数据结构定义。"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional, Tuple


@dataclass
class FuselageParams:
    """Fuselage 几何参数，从 SimpleRockets 2 XML 解析。"""

    length: float = 5.0
    radius_x: float = 2.0
    radius_z: float = 4.0
    offset_x: float = 0.0
    offset_y: float = 0.0
    offset_z: float = 0.0
    top_scale_x: float = 1.0
    top_scale_z: float = 1.0
    bottom_scale_x: float = 1.0
    bottom_scale_z: float = 1.0
    deformations: Tuple[float, float, float] = (0.0, 0.0, 0.0)
    corner_radiuses: Tuple[float, ...] = (1.0,) * 8
    clamp_distances: Tuple[float, ...] = (1.0,) * 8
    part_scale: Tuple[float, float, float] = (1.0, 1.0, 1.0)
    flatten_normals: str = ""  # "", "Top", "Bottom", "Both"

    @property
    def top_radius_x(self) -> float:
        return self.radius_x * self.top_scale_x

    @property
    def top_radius_z(self) -> float:
        return self.radius_z * self.top_scale_z

    @property
    def bottom_radius_x(self) -> float:
        return self.radius_x * self.bottom_scale_x

    @property
    def bottom_radius_z(self) -> float:
        return self.radius_z * self.bottom_scale_z

    @property
    def top_corners(self) -> Tuple[float, ...]:
        """顶部 4 个圆角参数。"""
        if len(self.corner_radiuses) >= 8:
            return self.corner_radiuses[:4]
        return (1.0, 1.0, 1.0, 1.0)

    @property
    def bottom_corners(self) -> Tuple[float, ...]:
        """底部 4 个圆角参数。"""
        if len(self.corner_radiuses) >= 8:
            return self.corner_radiuses[4:]
        return (1.0, 1.0, 1.0, 1.0)

    @property
    def top_deformation(self) -> float:
        return self.deformations[0] if len(self.deformations) >= 1 else 0.0

    @property
    def vertical_shear(self) -> float:
        return self.deformations[1] if len(self.deformations) >= 2 else 0.0

    @property
    def bottom_deformation(self) -> float:
        return self.deformations[2] if len(self.deformations) >= 3 else 0.0

    @property
    def top_clamp(self) -> Tuple[float, ...]:
        if len(self.clamp_distances) >= 8:
            return self.clamp_distances[:4]
        return (1.0, 1.0, 1.0, 1.0)

    @property
    def bottom_clamp(self) -> Tuple[float, ...]:
        if len(self.clamp_distances) >= 8:
            return self.clamp_distances[4:]
        return (1.0, 1.0, 1.0, 1.0)


@dataclass
class Material:
    """材质定义。"""

    name: str
    color: Tuple[float, float, float]  # RGB 0-1
    metallic: float  # 0-1
    roughness: float  # 0-1（已从游戏 smoothness 转换）


@dataclass
class PartData:
    """零件数据，在模块间传递。"""

    part_id: str = "unknown"
    part_type: str = "unknown"
    position: Tuple[float, float, float] = (0.0, 0.0, 0.0)
    rotation: Tuple[float, float, float] = (0.0, 0.0, 0.0)
    material_name: str = "default"
    material_index: int = 0
    fuselage_params: Optional[FuselageParams] = None
    wall_thickness: float = 0.0
    subdivisions: int = 5

    def to_dict(self) -> Dict[str, Any]:
        result: Dict[str, Any] = {
            "part_id": self.part_id,
            "part_type": self.part_type,
            "position": self.position,
            "rotation": self.rotation,
            "material_name": self.material_name,
            "material_index": self.material_index,
            "wall_thickness": self.wall_thickness,
            "subdivisions": self.subdivisions,
        }
        if self.fuselage_params:
            params = self.fuselage_params
            result["fuselage_params"] = {
                "length": params.length,
                "radius_x": params.radius_x,
                "radius_z": params.radius_z,
                "offset_x": params.offset_x,
                "offset_y": params.offset_y,
                "offset_z": params.offset_z,
                "top_scale_x": params.top_scale_x,
                "top_scale_z": params.top_scale_z,
                "bottom_scale_x": params.bottom_scale_x,
                "bottom_scale_z": params.bottom_scale_z,
                "deformations": params.deformations,
                "corner_radiuses": params.corner_radiuses,
                "clamp_distances": params.clamp_distances,
                "part_scale": params.part_scale,
                "flatten_normals": params.flatten_normals,
            }
        else:
            result["fuselage_params"] = None
        return result

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "PartData":
        part = cls(
            part_id=data.get("part_id", "unknown"),
            part_type=data.get("part_type", "unknown"),
            position=tuple(data.get("position", (0.0, 0.0, 0.0))),
            rotation=tuple(data.get("rotation", (0.0, 0.0, 0.0))),
            material_name=data.get("material_name", "default"),
            material_index=data.get("material_index", 0),
            wall_thickness=data.get("wall_thickness", 0.0),
            subdivisions=data.get("subdivisions", 5),
        )
        if data.get("fuselage_params"):
            fp = data["fuselage_params"]
            part.fuselage_params = FuselageParams(
                length=fp.get("length", 5.0),
                radius_x=fp.get("radius_x", 1.0),
                radius_z=fp.get("radius_z", 1.0),
                offset_x=fp.get("offset_x", 0.0),
                offset_y=fp.get("offset_y", 0.0),
                offset_z=fp.get("offset_z", 0.0),
                top_scale_x=fp.get("top_scale_x", 1.0),
                top_scale_z=fp.get("top_scale_z", 1.0),
                bottom_scale_x=fp.get("bottom_scale_x", 1.0),
                bottom_scale_z=fp.get("bottom_scale_z", 1.0),
                deformations=tuple(fp.get("deformations", (0.0, 0.0, 0.0))),
                corner_radiuses=tuple(fp.get("corner_radiuses", (1.0,) * 8)),
                clamp_distances=tuple(fp.get("clamp_distances", (1.0,) * 8)),
                part_scale=tuple(fp.get("part_scale", (1.0, 1.0, 1.0))),
                flatten_normals=fp.get("flatten_normals", ""),
            )
        return part


@dataclass
class MeshData:
    """网格数据，用于缓存和模块间传递。"""

    part_id: str
    vertices: List[Tuple[float, float, float]] = field(default_factory=list)
    normals: List[Tuple[float, float, float]] = field(default_factory=list)
    uvs: List[Tuple[float, float]] = field(default_factory=list)
    # face: (v1,v2,v3, vt1,vt2,vt3, vn1,vn2,vn3, material_name, part_id)
    faces: List[
        Tuple[int, int, int, int, int, int, int, int, int, str, str]
    ] = field(default_factory=list)

    def to_dict(self) -> Dict[str, Any]:
        return {
            "part_id": self.part_id,
            "vertices": self.vertices,
            "normals": self.normals,
            "uvs": self.uvs,
            "faces": self.faces,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "MeshData":
        mesh = cls(part_id=data["part_id"])
        mesh.vertices = [tuple(v) for v in data["vertices"]]
        mesh.normals = [tuple(n) for n in data["normals"]]
        mesh.uvs = [tuple(uv) for uv in data["uvs"]]
        mesh.faces = [tuple(f) for f in data["faces"]]
        return mesh


@dataclass
class RawGeometry:
    """零件模块生成的原始几何数据（裸几何，未变换/未做法线/未绑材质）。"""

    part_id: str
    material_name: str
    vertices: List[Tuple[float, float, float]] = field(default_factory=list)
    uvs: List[Tuple[float, float]] = field(default_factory=list)
    # 面数据 — 生成阶段为 8 值，NormalService 处理后变为 11 值
    # 8 值: (v1,v2,v3, vt1,vt2,vt3, material_name, part_id)
    # 11 值: (v1,v2,v3, vt1,vt2,vt3, vn1,vn2,vn3, material_name, part_id)
    faces: List[Tuple] = field(default_factory=list)
    # 环形顶点信息，供法线服务使用
    ring_info: Dict[str, Any] = field(default_factory=dict)
    # 为骨骼预留
    parent_part_id: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        return {
            "part_id": self.part_id,
            "material_name": self.material_name,
            "vertices": self.vertices,
            "uvs": self.uvs,
            "faces": self.faces,
            "ring_info": self.ring_info,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "RawGeometry":
        geom = cls(
            part_id=data["part_id"],
            material_name=data["material_name"],
        )
        geom.vertices = [tuple(v) for v in data["vertices"]]
        geom.uvs = [tuple(uv) for uv in data["uvs"]]
        geom.faces = [tuple(f) for f in data["faces"]]
        geom.ring_info = data.get("ring_info", {})
        return geom
