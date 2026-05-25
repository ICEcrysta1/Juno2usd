#!/usr/bin/env python3
"""TransformService: 对 RawGeometry 应用位置和旋转变换。"""

from __future__ import annotations

from typing import List, Tuple

import numpy as np

from ..models import RawGeometry
from ..utils.math3d import rotation_matrix


class TransformService:
    """位置/旋转变换服务。"""

    def apply_position(self, geom: RawGeometry, position: Tuple[float, float, float]) -> None:
        """将几何体的所有顶点平移。原地修改。

        Args:
            geom: 原始几何数据。
            position: 位移向量 (x, y, z)。
        """
        px, py, pz = position
        geom.vertices = [(v[0] + px, v[1] + py, v[2] + pz) for v in geom.vertices]

    def apply_rotation(
        self, geom: RawGeometry, rotation: Tuple[float, float, float]
    ) -> None:
        """旋转几何体（在应用位置之前调用）。原地修改。

        Args:
            geom: 原始几何数据。
            rotation: 欧拉角 (rx, ry, rz)，度数，Unity ZXY 顺序。
        """
        rot_mat = rotation_matrix(rotation)
        geom.vertices = [tuple(rot_mat @ np.array(v)) for v in geom.vertices]

    def apply_part_scale(
        self, coords: List[Tuple[float, float, float]], scale: Tuple[float, float, float]
    ) -> List[Tuple[float, float, float]]:
        """应用零件整体缩放。不原地修改。

        Args:
            coords: 坐标列表。
            scale: (sx, sy, sz) 缩放因子。

        Returns:
            缩放后的坐标列表。
        """
        sx, sy, sz = scale
        return [(x * sx, y * sy, z * sz) for x, y, z in coords]
