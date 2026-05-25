#!/usr/bin/env python3
"""NormalService: 通用法线计算服务。

纯数学执行器——根据 ring_info.smooth_groups 执行法线计算。
零件全权决定平滑组分配，NormalService 不猜测几何类型。
无 smooth_groups 时默认所有面同组（完全平滑）。
"""

from __future__ import annotations

import math
from collections import defaultdict
from typing import Dict, List, Tuple

from ..models import RawGeometry
from ..utils.constants import EPSILON
from ..utils.math3d import calc_face_normal


def _lookup_normal(
    nmap: Dict,
    v: int,
    fi: int,
    smooth_groups: dict,
) -> int:
    """从法线映射中查找顶点法线索引。

    以 (v, group_id) 双键查找（支持硬边多法线）。
    无 smooth_groups 时以 v 单键查找（fallback）。
    """
    if smooth_groups:
        gid = smooth_groups.get(fi, 0)
        return nmap.get((v, gid), -1)
    return nmap.get(v, -1)


class NormalService:
    """通用法线计算服务——纯数学执行器。"""

    def compute(self, geom: RawGeometry) -> List[Tuple[float, float, float]]:
        """计算法线。根据 ring_info.smooth_groups 执行。

        无 smooth_groups 时默认所有面同组（完全平滑）。
        原地修改 geom.faces 中的法线索引（8 值 → 11 值）。

        Returns:
            全局法线列表 [(nx, ny, nz), ...]。
        """
        ring_info = geom.ring_info or {}
        smooth_groups = ring_info.get("smooth_groups", {})

        if not geom.vertices or not geom.faces:
            return []

        normals: List[Tuple[float, float, float]] = []
        vertices = geom.vertices
        raw_faces = geom.faces

        all_faces = set(range(len(raw_faces)))
        if not smooth_groups:
            smooth_groups = {fi: 0 for fi in all_faces}

        # 按 (vertex, group_id) 累加法线
        nmap = self._normals_by_groups(
            vertices, raw_faces, all_faces, smooth_groups, normals
        )

        # 组装 11 值面
        faces_out: list = []
        for fi, face in enumerate(raw_faces):
            v1, v2, v3, vt1, vt2, vt3, mat, pid = face
            vn1 = _lookup_normal(nmap, v1, fi, smooth_groups)
            vn2 = _lookup_normal(nmap, v2, fi, smooth_groups)
            vn3 = _lookup_normal(nmap, v3, fi, smooth_groups)

            if vn1 < 0 or vn2 < 0 or vn3 < 0:
                vn_new = self._add_face_normal(vertices, v1, v2, v3, normals)
                if vn1 < 0:
                    vn1 = vn_new
                if vn2 < 0:
                    vn2 = vn_new
                if vn3 < 0:
                    vn3 = vn_new

            faces_out.append((v1, v2, v3, vt1, vt2, vt3, vn1, vn2, vn3, mat, pid))

        geom.faces = faces_out
        return normals

    def _normals_by_groups(
        self,
        vertices: List[Tuple[float, float, float]],
        raw_faces: List[Tuple],
        face_indices: set,
        smooth_groups: Dict[int, int],
        normals: List[Tuple[float, float, float]],
        invert: bool = False,
    ) -> Dict[Tuple[int, int], int]:
        """按 (vertex, group_id) 累加面法线并归一化。

        同一 (顶点, group_id) 内平均面法线，不同 group 间法线拆分（硬边）。

        Returns:
            (vertex, group_id) → 法线索引的映射。
        """
        vg_acc: Dict[Tuple[int, int], List[float]] = defaultdict(
            lambda: [0.0, 0.0, 0.0]
        )

        for fi in face_indices:
            gid = smooth_groups.get(fi, 0)
            f = raw_faces[fi]
            fn = calc_face_normal(vertices, f[0], f[1], f[2])
            if invert:
                fn = (-fn[0], -fn[1], -fn[2])
            for corner in range(3):
                v = f[corner]
                key = (v, gid)
                vg_acc[key][0] += fn[0]
                vg_acc[key][1] += fn[1]
                vg_acc[key][2] += fn[2]

        result: Dict[Tuple[int, int], int] = {}
        for (v, gid), acc in vg_acc.items():
            length = math.sqrt(acc[0] ** 2 + acc[1] ** 2 + acc[2] ** 2)
            if length > EPSILON:
                fn = (acc[0] / length, acc[1] / length, acc[2] / length)
            else:
                fn = (0.0, 1.0, 0.0)
            result[(v, gid)] = len(normals)
            normals.append(fn)

        return result

    @staticmethod
    def _add_face_normal(
        vertices: List[Tuple[float, float, float]],
        v1: int,
        v2: int,
        v3: int,
        normals: List[Tuple[float, float, float]],
    ) -> int:
        """计算面法线并添加到法线列表。返回法线索引。"""
        fn = calc_face_normal(vertices, v1, v2, v3)
        normals.append(fn)
        return len(normals) - 1
