#!/usr/bin/env python3
"""XML 解析模块：解析 SimpleRockets 2 XML，提取零件和材质。"""

from __future__ import annotations

import json
import defusedxml.ElementTree as ET
from typing import Any, Dict, List, Optional, Tuple

from .models import FuselageParams, Material, PartData
from .utils.math3d import parse_scale, parse_vector
from .utils.naming import clean_material_name


# ---- XML 解析工具函数 ----

def _parse_fuselage_xml(fuselage_elem: ET.Element) -> FuselageParams:
    """从 XML Fuselage 元素构建 FuselageParams。"""
    params = FuselageParams()

    if "offset" in fuselage_elem.attrib:
        offset = parse_vector(fuselage_elem.attrib["offset"])
        params.offset_x = offset[0]
        params.offset_y = offset[1]
        params.offset_z = offset[2]

    if "topScale" in fuselage_elem.attrib:
        scale = parse_scale(fuselage_elem.attrib["topScale"])
        params.top_scale_x = scale[0]
        params.top_scale_z = scale[1]

    if "bottomScale" in fuselage_elem.attrib:
        scale = parse_scale(fuselage_elem.attrib["bottomScale"])
        params.bottom_scale_x = scale[0]
        params.bottom_scale_z = scale[1]

    if "deformations" in fuselage_elem.attrib:
        params.deformations = parse_vector(fuselage_elem.attrib["deformations"])

    if "cornerRadiuses" in fuselage_elem.attrib:
        params.corner_radiuses = tuple(
            float(x) for x in fuselage_elem.attrib["cornerRadiuses"].split(",")
        )

    if "clampDistances" in fuselage_elem.attrib:
        params.clamp_distances = tuple(
            float(x) for x in fuselage_elem.attrib["clampDistances"].split(",")
        )

    return params


def _parse_materials(root: ET.Element) -> List[Material]:
    """从 XML 根元素解析材质列表。"""
    materials: List[Material] = []
    theme = root.find(".//DesignerSettings/Theme")
    if theme is None:
        theme = root.find(".//Themes/Theme")

    if theme is not None:
        for mat_elem in theme.findall("Material"):
            name = mat_elem.get("name", "Unknown")
            color_hex = mat_elem.get("color", "FFFFFF")
            metallic = float(mat_elem.get("m", "0"))
            smoothness = float(mat_elem.get("s", "0"))

            try:
                r = int(color_hex[0:2], 16) / 255.0
                g = int(color_hex[2:4], 16) / 255.0
                b = int(color_hex[4:6], 16) / 255.0
            except (ValueError, IndexError):
                r, g, b = 1.0, 1.0, 1.0

            roughness = 1.0 - smoothness
            materials.append(Material(name, (r, g, b), metallic, roughness))

    return materials


def _get_material_index(part_elem: ET.Element) -> int:
    """获取零件使用的材质索引。"""
    materials_attr = part_elem.get("materials", "0,0,0,0,0")
    try:
        indices = [int(x) for x in materials_attr.split(",")]
        return indices[0] if indices else 0
    except (ValueError, IndexError):
        return 0


def _safe_parse_vector(text: str) -> Tuple[float, float, float]:
    """安全解析向量字符串。"""
    try:
        parts = text.split(",")
        return (float(parts[0]), float(parts[1]), float(parts[2]))
    except (ValueError, TypeError):
        return (0.0, 0.0, 0.0)


# ---- 解析器类 ----

class SR2XMLParser:
    """SR2 XML 解析器。"""

    def __init__(self, default_radius_x: float = 1.0, default_radius_z: float = 1.0) -> None:
        self.default_radius_x = default_radius_x
        self.default_radius_z = default_radius_z
        self.materials: List[Material] = []
        self.parts: List[PartData] = []

    def parse_file(self, xml_file: str) -> Dict[str, Any]:
        """解析 XML 文件。

        Returns:
            {"materials": [...], "parts": [...], "metadata": {...}}
        """
        print(f"[Parser] 解析XML文件: {xml_file}")
        tree = ET.parse(xml_file)
        root = tree.getroot()

        self.materials = _parse_materials(root)
        print(f"[Parser] 解析到 {len(self.materials)} 种材质")

        xml_parts = root.findall(".//Part")
        print(f"[Parser] 找到 {len(xml_parts)} 个部件")

        self.parts = []
        for part_elem in xml_parts:
            part_data = self._parse_part(part_elem)
            if part_data:
                self.parts.append(part_data)

        print(f"[Parser] 成功解析 {len(self.parts)} 个有效部件")
        return {
            "materials": self.materials,
            "parts": self.parts,
            "metadata": {
                "source_file": xml_file,
                "default_radius_x": self.default_radius_x,
                "default_radius_z": self.default_radius_z,
                "total_parts": len(self.parts),
                "total_materials": len(self.materials),
            },
        }

    def _parse_part(self, part_elem: ET.Element) -> Optional[PartData]:
        """解析单个零件元素。"""
        part_id = part_elem.get("id", "unknown")
        part_type = part_elem.get("partType", "unknown")

        position = _safe_parse_vector(part_elem.get("position", "0,0,0"))
        rotation = _safe_parse_vector(part_elem.get("rotation", "0,0,0"))

        mat_idx = _get_material_index(part_elem)
        if mat_idx < len(self.materials):
            mat_name = clean_material_name(self.materials[mat_idx].name)
        else:
            mat_name = "default"

        part_data = PartData(
            part_id=part_id,
            part_type=part_type,
            position=position,
            rotation=rotation,
            material_name=mat_name,
            material_index=mat_idx,
        )

        if part_type in ("Fuselage1", "Strut1"):
            self._parse_fuselage(part_elem, part_data)
        elif part_type in ("Inlet1", "FairingBase1", "Fairing1"):
            self._parse_hollow_cylinder(part_elem, part_data)
        elif part_type == "NoseCone1":
            self._parse_cone(part_elem, part_data, is_hollow=False)
        elif part_type == "FairingNoseCone1":
            self._parse_cone(part_elem, part_data, is_hollow=True)
        elif part_type in ("Rotator1", "HingeRotator1"):
            pass  # 关节零件，不需要几何参数
        else:
            return None

        return part_data

    def _parse_fuselage(self, part_elem: ET.Element, part_data: PartData) -> None:
        """解析 Fuselage1 / Strut1 的 Fuselage 参数。"""
        fuselage = part_elem.find("Fuselage")
        if fuselage is None:
            return

        params = self._build_fuselage_params(fuselage)

        if part_data.part_type == "Strut1" and "cornerRadiuses" not in fuselage.attrib:
            params.corner_radiuses = (0.5,) * 8

        part_data.fuselage_params = params
        print(f"[Parser]   {part_data.part_type} {part_data.part_id}: "
              f"length={params.length:.3f}, "
              f"radius=({params.radius_x:.2f}, {params.radius_z:.2f})")

    def _parse_hollow_cylinder(self, part_elem: ET.Element, part_data: PartData) -> None:
        """解析 Inlet1 / FairingBase1 / Fairing1。"""
        fuselage = part_elem.find("Fuselage")
        if fuselage is None:
            return

        params = self._build_fuselage_params(fuselage)
        part_data.fuselage_params = params
        part_data.wall_thickness = 0.045
        print(f"[Parser]   {part_data.part_type} {part_data.part_id}: "
              f"length={params.length:.3f}, wall={part_data.wall_thickness:.3f}")

    def _parse_cone(self, part_elem: ET.Element, part_data: PartData, *, is_hollow: bool) -> None:
        """解析 NoseCone1 / FairingNoseCone1。"""
        fuselage = part_elem.find("Fuselage")
        if fuselage is None:
            return

        params = self._build_fuselage_params(fuselage)
        part_data.fuselage_params = params
        part_data.subdivisions = 5
        if is_hollow:
            part_data.wall_thickness = 0.045
        print(f"[Parser]   {part_data.part_type} {part_data.part_id}: "
              f"length={params.length:.3f}, subdivisions={part_data.subdivisions}")

    def _build_fuselage_params(self, fuselage: ET.Element) -> FuselageParams:
        """从 XML Fuselage 元素构建 FuselageParams（含 partScale）。"""
        params = _parse_fuselage_xml(fuselage)
        params.radius_x = self.default_radius_x
        params.radius_z = self.default_radius_z
        params.length = params.offset_y * 2

        if params.length < 0.001:
            print(f"[Parser] 警告: {part_data.part_id} offset_y≈0，长度设为默认 1.0")
            params.length = 1.0

        config = fuselage.find("../Config")
        if config is not None and "partScale" in config.attrib:
            params.part_scale = tuple(
                float(x) for x in config.attrib.get("partScale", "1,1,1").split(",")
            )

        return params

    def save_to_json(self, output_file: str) -> str:
        """保存解析结果到 JSON 文件。"""
        data = {
            "metadata": {
                "default_radius_x": self.default_radius_x,
                "default_radius_z": self.default_radius_z,
                "total_parts": len(self.parts),
                "total_materials": len(self.materials),
            },
            "materials": [
                {
                    "name": m.name,
                    "clean_name": clean_material_name(m.name),
                    "color": m.color,
                    "metallic": m.metallic,
                    "roughness": m.roughness,
                }
                for m in self.materials
            ],
            "parts": [p.to_dict() for p in self.parts],
        }
        with open(output_file, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2, ensure_ascii=False)
        print(f"[Parser] 解析结果已保存到: {output_file}")
        return output_file
