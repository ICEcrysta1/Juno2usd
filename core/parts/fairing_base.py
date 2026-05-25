#!/usr/bin/env python3
"""FairingBase1 零件模块：空心倒置圆柱。"""

from __future__ import annotations

from typing import Optional

from .base import PartBase
from ._geometry import generate_cylinder_geometry
from ..models import FuselageParams, RawGeometry


class FairingBasePart(PartBase):
    """FairingBase1 — 空心倒置整流罩底座。

    与其他空心零件不同，FairingBase1 的端盖法线方向是反的。
    法线方向由 NormalService 根据 part_type 自动处理。
    """

    DEFAULT_WALL_THICKNESS: float = 0.045

    def generate(self, params: FuselageParams, segments: int) -> RawGeometry:
        return generate_cylinder_geometry(
            part_id="",
            material_name="",
            part_type="FairingBase1",
            params=params,
            segments=segments,
            is_hollow=True,
            wall_thickness=self.DEFAULT_WALL_THICKNESS,
        )

    def find_parent(self, part_id: str, connections: dict[str, Optional[str]]) -> Optional[str]:
        return connections.get(part_id)
