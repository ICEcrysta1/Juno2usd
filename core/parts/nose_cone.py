#!/usr/bin/env python3
"""NoseCone1 零件模块：实心锥体，5 段纵向细分。"""

from __future__ import annotations

from typing import Optional

from .base import PartBase
from ._geometry import generate_cone_geometry
from ..models import FuselageParams, RawGeometry


class NoseConePart(PartBase):
    """NoseCone1 — 实心鼻锥。"""

    DEFAULT_SUBDIVISIONS: int = 5

    def generate(self, params: FuselageParams, segments: int) -> RawGeometry:
        bottom_flat = params.flatten_normals in ("Bottom", "Both")
        return generate_cone_geometry(
            part_id="",
            material_name="",
            part_type="NoseCone1",
            params=params,
            segments=segments,
            is_hollow=False,
            subdivisions=self.DEFAULT_SUBDIVISIONS,
            smooth_policy={
                "outer_side": "angle_split",
                "bottom_cap": "independent" if bottom_flat else "merge_outer_side",
            },
        )

    def find_parent(self, part_id: str, connections: dict[str, Optional[str]]) -> Optional[str]:
        return connections.get(part_id)
