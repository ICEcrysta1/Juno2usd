#!/usr/bin/env python3
"""Fairing1 零件模块：空心圆柱。"""

from __future__ import annotations

from typing import Optional

from .base import PartBase
from ._geometry import generate_cylinder_geometry
from ..models import FuselageParams, RawGeometry


class FairingPart(PartBase):
    """Fairing1 — 空心整流罩主体。"""

    DEFAULT_WALL_THICKNESS: float = 0.045

    def generate(self, params: FuselageParams, segments: int) -> RawGeometry:
        return generate_cylinder_geometry(
            part_id="",
            material_name="",
            part_type="Fairing1",
            params=params,
            segments=segments,
            is_hollow=True,
            wall_thickness=self.DEFAULT_WALL_THICKNESS,
        )

    def find_parent(self, part_id: str, connections: dict[str, Optional[str]]) -> Optional[str]:
        return connections.get(part_id)
