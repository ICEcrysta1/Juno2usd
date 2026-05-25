#!/usr/bin/env python3
"""Fuselage1 零件模块：实心圆角矩形圆柱体。"""

from __future__ import annotations

from typing import Optional

from .base import PartBase
from ._geometry import generate_cylinder_geometry
from ..models import FuselageParams, RawGeometry


class FuselagePart(PartBase):
    """Fuselage1 — 标准实心圆柱。"""

    def generate(self, params: FuselageParams, segments: int) -> RawGeometry:
        bottom_flat = params.flatten_normals in ("Bottom", "Both")
        top_flat = params.flatten_normals in ("Top", "Both")
        return generate_cylinder_geometry(
            part_id="",
            material_name="",
            part_type="Fuselage1",
            params=params,
            segments=segments,
            is_hollow=False,
            smooth_policy={
                "outer_side": "angle_split",
                "bottom_cap": "independent" if bottom_flat else "merge_outer_side",
                "top_cap": "independent" if top_flat else "merge_outer_side",
            },
        )

    def find_parent(self, part_id: str, connections: dict[str, Optional[str]]) -> Optional[str]:
        return connections.get(part_id)
