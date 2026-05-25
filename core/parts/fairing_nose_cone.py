#!/usr/bin/env python3
"""FairingNoseCone1 零件模块：空心锥体，5 段纵向细分，壁厚 0.045。"""

from __future__ import annotations

from typing import Optional

from .base import PartBase
from ._geometry import generate_cone_geometry
from ..models import FuselageParams, RawGeometry


class FairingNoseConePart(PartBase):
    """FairingNoseCone1 — 空心整流罩鼻锥。"""

    DEFAULT_SUBDIVISIONS: int = 5
    DEFAULT_WALL_THICKNESS: float = 0.045

    def generate(self, params: FuselageParams, segments: int) -> RawGeometry:
        return generate_cone_geometry(
            part_id="",
            material_name="",
            part_type="FairingNoseCone1",
            params=params,
            segments=segments,
            is_hollow=True,
            wall_thickness=self.DEFAULT_WALL_THICKNESS,
            subdivisions=self.DEFAULT_SUBDIVISIONS,
        )

    def find_parent(self, part_id: str, connections: dict[str, Optional[str]]) -> Optional[str]:
        return connections.get(part_id)
