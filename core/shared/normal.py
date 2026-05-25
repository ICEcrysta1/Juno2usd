#!/usr/bin/env python3
"""NormalService: 统一法线计算服务。

根据 ring_info 判断几何类型，调度到对应的法线计算方法。
支持: 圆柱体（侧面+端盖）、鼻锥（多环）、硬边拆分、角度自适应。
"""

from __future__ import annotations

import math
from collections import defaultdict
from typing import Dict, List, Optional, Set, Tuple

from ..models import RawGeometry
from ..utils.constants import (
    CAP_SMOOTH_DEFAULT_DEG,
    CAP_SMOOTH_HIGH_CORNER_DEG,
    CAP_SMOOTH_LOW_CORNER_DEG,
    EPSILON,
    HARD_EDGE_ANGLE_DEG,
)


def _lookup_normal(
    nmap: Dict,
    v: int,
    fi: int,
    smooth_groups: dict,
) -> int:
    """从法线映射中查找顶点法线索引。

    当 smooth_groups 可用时，以 (v, group_id) 双键查找（支持硬边多法线）。
    否则以 v 单键查找（fallback 模式）。
    """
    if smooth_groups:
        gid = smooth_groups.get(fi, 0)
        return nmap.get((v, gid), -1)
    return nmap.get(v, -1)


class NormalService:
    """统一法线计算服务。"""

    def compute(self, geom: RawGeometry) -> List[Tuple[float, float, float]]:
        """计算 RawGeometry 的法线。原地修改 geom.faces 中的法线索引。

        Args:
            geom: 原始几何数据（含 ring_info）。

        Returns:
            全局法线列表 [(nx, ny, nz), ...]。
        """
        ring_info = geom.ring_info

        if not ring_info or not geom.vertices:
            return self._compute_simple(geom)

        if ring_info.get("is_nose_cone"):
            return self._compute_cone(geom, ring_info)

        return self._compute_cylinder(geom, ring_info)

    # ---- 公共入口（按几何类型） ----

    def _compute_cylinder(
        self, geom: RawGeometry, ring_info: dict
    ) -> List[Tuple[float, float, float]]:
        """计算圆柱体（含空心）的法线。"""
        normals: List[Tuple[float, float, float]] = []

        segments = ring_info.get("segments", 24)
        bottom_indices: List[int] = ring_info.get("bottom_indices", [])
        top_indices: List[int] = ring_info.get("top_indices", [])
        bottom_inner: Optional[List[int]] = ring_info.get("bottom_inner_indices")
        top_inner: Optional[List[int]] = ring_info.get("top_inner_indices")
        bottom_corners = ring_info.get("bottom_corners", (1.0, 1.0, 1.0, 1.0))
        top_corners = ring_info.get("top_corners", (1.0, 1.0, 1.0, 1.0))
        is_inlet = ring_info.get("is_inlet", False)
        part_type = ring_info.get("part_type", "")
        smooth_groups = ring_info.get("smooth_groups", {})

        vertices = geom.vertices
        raw_faces = geom.faces

        # 构建顶点集
        bot_set = set(bottom_indices)
        top_set = set(top_indices)
        bot_in_set = set(bottom_inner) if bottom_inner else set()
        top_in_set = set(top_inner) if top_inner else set()

        # 分类面
        outer_side: Set[int] = set()
        outer_cap_bot: Set[int] = set()
        outer_cap_top: Set[int] = set()
        inner_side: Set[int] = set()
        inner_cap_bot: Set[int] = set()
        inner_cap_top: Set[int] = set()

        for fi, face in enumerate(raw_faces):
            verts = {face[0], face[1], face[2]}

            if is_inlet and bottom_inner and top_inner:
                if verts.issubset(bot_set | bot_in_set):
                    if verts & bot_set:
                        outer_cap_bot.add(fi)
                    else:
                        inner_cap_bot.add(fi)
                elif verts.issubset(top_set | top_in_set):
                    if verts & top_set:
                        outer_cap_top.add(fi)
                    else:
                        inner_cap_top.add(fi)
                elif verts.issubset(bot_set | top_set):
                    outer_side.add(fi)
                elif verts.issubset(bot_in_set | top_in_set):
                    inner_side.add(fi)
            else:
                if verts.issubset(bot_set) and len(verts & bot_set) >= 2:
                    outer_cap_bot.add(fi)
                elif verts.issubset(top_set) and len(verts & top_set) >= 2:
                    outer_cap_top.add(fi)
                elif verts.issubset(bot_set | top_set):
                    outer_side.add(fi)

        # 侧面法线 — 优先使用零件提供的 smooth_groups
        if smooth_groups:
            side_map = self._normals_by_groups(
                vertices, raw_faces, outer_side, smooth_groups, normals
            )
        else:
            side_map = self._angle_split_normals(
                vertices, raw_faces, outer_side, normals
            )
        inner_side_map: Dict[int, int] = {}
        if is_inlet and inner_side:
            if smooth_groups:
                inner_side_map = self._normals_by_groups(
                    vertices, raw_faces, inner_side, smooth_groups, normals
                )
            else:
                inner_side_map = self._angle_split_normals(
                    vertices, raw_faces, inner_side, normals
                )

        # 端盖法线（圆柱端盖，径向）
        cap_map: Dict[int, int] = {}
        if bottom_indices:
            caps = self._compute_cap_radial(
                [vertices[i] for i in bottom_indices], bottom_corners, is_top=False
            )
            for i, idx in enumerate(bottom_indices):
                vn = len(normals)
                normals.append(caps[i])
                cap_map[idx] = vn
        if top_indices:
            caps = self._compute_cap_radial(
                [vertices[i] for i in top_indices], top_corners, is_top=True
            )
            for i, idx in enumerate(top_indices):
                vn = len(normals)
                normals.append(caps[i])
                cap_map[idx] = vn

        # 内层端盖
        inner_cap_map: Dict[int, int] = {}
        if is_inlet and bottom_inner and top_inner:
            ibc = [vertices[i] for i in bottom_inner]
            itc = [vertices[i] for i in top_inner]
            for i, idx in enumerate(bottom_inner):
                vn = len(normals)
                normals.append(self._compute_cap_radial(ibc, bottom_corners, is_top=False, invert=True)[i])
                inner_cap_map[idx] = vn
            for i, idx in enumerate(top_inner):
                vn = len(normals)
                normals.append(self._compute_cap_radial(itc, top_corners, is_top=True, invert=True)[i])
                inner_cap_map[idx] = vn

        # 端盖轴向法线（空心零件 Y 方向法线）
        outer_cap_bot_nm: Dict[int, int] = {}
        outer_cap_top_nm: Dict[int, int] = {}
        inner_cap_bot_nm: Dict[int, int] = {}
        inner_cap_top_nm: Dict[int, int] = {}

        if is_inlet and bottom_inner and top_inner:
            if part_type in ("FairingBase1", "Inlet1"):
                obn, ibn = (0.0, -1.0, 0.0), (0.0, -1.0, 0.0)
                otn, itn = (0.0, 1.0, 0.0), (0.0, 1.0, 0.0)
            else:
                obn, ibn = (0.0, 1.0, 0.0), (0.0, 1.0, 0.0)
                otn, itn = (0.0, -1.0, 0.0), (0.0, -1.0, 0.0)

            for idx in bottom_indices:
                vn = len(normals)
                normals.append(obn)
                outer_cap_bot_nm[idx] = vn
            for idx in bottom_inner:
                vn = len(normals)
                normals.append(ibn)
                inner_cap_bot_nm[idx] = vn
            for idx in top_indices:
                vn = len(normals)
                normals.append(otn)
                outer_cap_top_nm[idx] = vn
            for idx in top_inner:
                vn = len(normals)
                normals.append(itn)
                inner_cap_top_nm[idx] = vn

        # 组装面
        faces_out: list = []
        for fi, face in enumerate(raw_faces):
            v1, v2, v3, vt1, vt2, vt3, mat, pid = face

            if is_inlet and bottom_inner and top_inner:
                if fi in outer_cap_bot:
                    vn1 = outer_cap_bot_nm.get(v1, cap_map.get(v1, -1))
                    vn2 = outer_cap_bot_nm.get(v2, cap_map.get(v2, -1))
                    vn3 = outer_cap_bot_nm.get(v3, cap_map.get(v3, -1))
                elif fi in outer_cap_top:
                    vn1 = outer_cap_top_nm.get(v1, cap_map.get(v1, -1))
                    vn2 = outer_cap_top_nm.get(v2, cap_map.get(v2, -1))
                    vn3 = outer_cap_top_nm.get(v3, cap_map.get(v3, -1))
                elif fi in inner_cap_bot:
                    vn1 = inner_cap_bot_nm.get(v1, inner_cap_map.get(v1, -1))
                    vn2 = inner_cap_bot_nm.get(v2, inner_cap_map.get(v2, -1))
                    vn3 = inner_cap_bot_nm.get(v3, inner_cap_map.get(v3, -1))
                elif fi in inner_cap_top:
                    vn1 = inner_cap_top_nm.get(v1, inner_cap_map.get(v1, -1))
                    vn2 = inner_cap_top_nm.get(v2, inner_cap_map.get(v2, -1))
                    vn3 = inner_cap_top_nm.get(v3, inner_cap_map.get(v3, -1))
                elif fi in outer_side:
                    vn1 = _lookup_normal(side_map, v1, fi, smooth_groups)
                    vn2 = _lookup_normal(side_map, v2, fi, smooth_groups)
                    vn3 = _lookup_normal(side_map, v3, fi, smooth_groups)
                elif fi in inner_side:
                    vn1 = _lookup_normal(inner_side_map, v1, fi, smooth_groups)
                    vn2 = _lookup_normal(inner_side_map, v2, fi, smooth_groups)
                    vn3 = _lookup_normal(inner_side_map, v3, fi, smooth_groups)
                else:
                    vn1 = vn2 = vn3 = -1
            else:
                vn1 = _lookup_normal(side_map, v1, fi, smooth_groups)
                vn2 = _lookup_normal(side_map, v2, fi, smooth_groups)
                vn3 = _lookup_normal(side_map, v3, fi, smooth_groups)

            if vn1 < 0:
                vn1 = vn2 = vn3 = self._add_face_normal(vertices, v1, v2, v3, normals)
            elif vn2 < 0 or vn3 < 0:
                vn_fb = self._add_face_normal(vertices, v1, v2, v3, normals)
                if vn2 < 0:
                    vn2 = vn_fb
                if vn3 < 0:
                    vn3 = vn_fb

            faces_out.append((v1, v2, v3, vt1, vt2, vt3, vn1, vn2, vn3, mat, pid))

        geom.faces = faces_out
        return normals

    def _compute_cone(
        self, geom: RawGeometry, ring_info: dict
    ) -> List[Tuple[float, float, float]]:
        """计算鼻锥（多环结构）的法线，带拆边硬边。"""
        normals: List[Tuple[float, float, float]] = []

        segments = ring_info.get("segments", 24)
        ring_indices: List[List[int]] = ring_info.get("ring_indices", [])
        inner_ring_indices: Optional[List[List[int]]] = ring_info.get("inner_ring_indices")
        is_hollow = ring_info.get("is_hollow", False)
        has_bottom_cap = ring_info.get("has_bottom_cap", True)
        smooth_groups = ring_info.get("smooth_groups", {})

        vertices = geom.vertices
        raw_faces = geom.faces

        if not ring_indices or len(ring_indices) < 2:
            return self._compute_simple(geom)

        bottom_ring = set(ring_indices[0])
        top_ring = set(ring_indices[-1])
        inner_bottom = set(inner_ring_indices[0]) if inner_ring_indices and is_hollow else set()
        inner_top = set(inner_ring_indices[-1]) if inner_ring_indices and is_hollow else set()

        # 找到中心顶点
        all_verts = set()
        for ring in ring_indices:
            all_verts.update(ring)
        if inner_ring_indices:
            for ring in inner_ring_indices:
                all_verts.update(ring)
        max_ring_v = max(all_verts) if all_verts else -1

        center_vertex: Optional[int] = None
        for face in raw_faces:
            for v in face[:3]:
                if v > max_ring_v:
                    center_vertex = v
                    break
            if center_vertex is not None:
                break

        all_outer_rings = [set(r) for r in ring_indices]
        all_inner_rings = [set(r) for r in inner_ring_indices] if inner_ring_indices and is_hollow else []

        # 分类面
        side_set: Set[int] = set()
        cap_bot_set: Set[int] = set()
        inner_side_set: Set[int] = set()
        inner_cap_bot_set: Set[int] = set()

        for fi, face in enumerate(raw_faces):
            verts = {face[0], face[1], face[2]}

            if is_hollow and inner_ring_indices:
                if verts.issubset(bottom_ring | inner_bottom):
                    if verts & bottom_ring and verts & inner_bottom:
                        cap_bot_set.add(fi)
                    else:
                        side_set.add(fi)
                elif any(verts & ring for ring in all_outer_rings):
                    side_set.add(fi)
                elif any(verts & ring for ring in all_inner_rings):
                    inner_side_set.add(fi)
            else:
                if center_vertex is not None and center_vertex in verts:
                    cap_bot_set.add(fi)
                elif verts.issubset(bottom_ring):
                    cap_bot_set.add(fi)
                else:
                    side_set.add(fi)

        # 侧面法线 — 优先使用零件提供的 smooth_groups
        if smooth_groups:
            side_nm = self._normals_by_groups(vertices, raw_faces, side_set, smooth_groups, normals)
            cap_nm = self._normals_by_groups(vertices, raw_faces, cap_bot_set, smooth_groups, normals)
        else:
            side_nm = self._smooth_group_normals(vertices, raw_faces, side_set, normals)
            cap_nm = self._smooth_group_normals(vertices, raw_faces, cap_bot_set, normals)

        inner_side_nm: Dict[int, int] = {}
        inner_cap_nm: Dict[int, int] = {}
        if is_hollow and inner_ring_indices:
            if smooth_groups:
                inner_side_nm = self._normals_by_groups(vertices, raw_faces, inner_side_set, smooth_groups, normals, invert=True)
                inner_cap_nm = self._normals_by_groups(vertices, raw_faces, inner_cap_bot_set, smooth_groups, normals)
            else:
                inner_side_nm = self._smooth_group_normals(vertices, raw_faces, inner_side_set, normals, invert=True)
                inner_cap_nm = self._smooth_group_normals(vertices, raw_faces, inner_cap_bot_set, normals)

        # 组装面
        faces_out: list = []
        for fi, face in enumerate(raw_faces):
            v1, v2, v3, vt1, vt2, vt3, mat, pid = face

            if fi in cap_bot_set:
                vn1 = _lookup_normal(cap_nm, v1, fi, smooth_groups)
                vn2 = _lookup_normal(cap_nm, v2, fi, smooth_groups)
                vn3 = _lookup_normal(cap_nm, v3, fi, smooth_groups)
            elif fi in side_set:
                vn1 = _lookup_normal(side_nm, v1, fi, smooth_groups)
                vn2 = _lookup_normal(side_nm, v2, fi, smooth_groups)
                vn3 = _lookup_normal(side_nm, v3, fi, smooth_groups)
            elif fi in inner_cap_bot_set:
                vn1 = _lookup_normal(inner_cap_nm, v1, fi, smooth_groups)
                vn2 = _lookup_normal(inner_cap_nm, v2, fi, smooth_groups)
                vn3 = _lookup_normal(inner_cap_nm, v3, fi, smooth_groups)
            elif fi in inner_side_set:
                vn1 = _lookup_normal(inner_side_nm, v1, fi, smooth_groups)
                vn2 = _lookup_normal(inner_side_nm, v2, fi, smooth_groups)
                vn3 = _lookup_normal(inner_side_nm, v3, fi, smooth_groups)
            else:
                vn1 = vn2 = vn3 = -1

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

    # ---- 端盖径向法线 ----

    def _compute_cap_radial(
        self,
        coords: List[Tuple[float, float, float]],
        corner_radii: Tuple[float, ...],
        is_top: bool = True,
        invert: bool = False,
    ) -> List[Tuple[float, float, float]]:
        """计算端盖径向法线（自适应圆滑度）。"""
        n = len(coords)
        if n == 0:
            return []

        cx = sum(c[0] for c in coords) / n
        cz = sum(c[2] for c in coords) / n

        avg_corner = 0.5
        if corner_radii:
            avg_corner = sum(corner_radii) / len(corner_radii)

        if avg_corner < 0.1:
            smooth_thresh = math.radians(CAP_SMOOTH_LOW_CORNER_DEG)
        elif avg_corner > 0.9:
            smooth_thresh = math.radians(CAP_SMOOTH_HIGH_CORNER_DEG)
        else:
            smooth_thresh = math.radians(CAP_SMOOTH_DEFAULT_DEG)

        results: List[Tuple[float, float, float]] = []

        for i in range(n):
            prev_i = (i - 1) % n
            next_i = (i + 1) % n

            vc = coords[i]
            vp = coords[prev_i]
            vn = coords[next_i]

            ep = (vp[0] - vc[0], vp[2] - vc[2])
            en = (vn[0] - vc[0], vn[2] - vc[2])
            lp = math.sqrt(ep[0] ** 2 + ep[1] ** 2)
            ln = math.sqrt(en[0] ** 2 + en[1] ** 2)

            if lp > EPSILON and ln > EPSILON:
                epn = (ep[0] / lp, ep[1] / lp)
                enn = (en[0] / ln, en[1] / ln)
                dot = max(-1.0, min(1.0, epn[0] * enn[0] + epn[1] * enn[1]))
                angle = math.acos(dot)
            else:
                angle = math.pi

            if angle >= smooth_thresh:
                dx, dz = cx - vc[0], cz - vc[2]
                lh = math.sqrt(dx**2 + dz**2)
                nx = dx / lh if lh > EPSILON else 0.0
                nz = dz / lh if lh > EPSILON else 0.0
            else:
                bx = -(epn[0] + enn[0])
                bz = -(epn[1] + enn[1])
                bl = math.sqrt(bx**2 + bz**2)
                if bl > EPSILON:
                    nx, nz = bx / bl, bz / bl
                else:
                    dx, dz = cx - vc[0], cz - vc[2]
                    lh = math.sqrt(dx**2 + dz**2)
                    nx = dx / lh if lh > EPSILON else 0.0
                    nz = dz / lh if lh > EPSILON else 0.0

            if 0.01 < avg_corner < 0.99:
                dx, dz = cx - vc[0], cz - vc[2]
                lh = math.sqrt(dx**2 + dz**2)
                if lh > EPSILON:
                    rnx, rnz = dx / lh, dz / lh
                else:
                    rnx, rnz = nx, nz
                nx = nx * (1 - avg_corner) + rnx * avg_corner
                nz = nz * (1 - avg_corner) + rnz * avg_corner
                nl = math.sqrt(nx**2 + nz**2)
                if nl > EPSILON:
                    nx, nz = nx / nl, nz / nl

            ny = 1.0 if is_top else -1.0
            if invert:
                nx, ny, nz = -nx, -ny, -nz

            results.append((nx, ny, nz))

        return results

    # ---- 硬边拆分法线（角度自适应） ----

    def _angle_split_normals(
        self,
        vertices: List[Tuple[float, float, float]],
        raw_faces: List[Tuple],
        face_indices: Set[int],
        normals: List[Tuple[float, float, float]],
        angle_threshold: float = HARD_EDGE_ANGLE_DEG,
    ) -> Dict[int, int]:
        """基于角度检测计算侧面法线，在锐边处拆开。

        Returns:
            顶点索引 -> 法线索引的映射。
        """
        if not face_indices:
            return {}

        threshold_rad = math.radians(angle_threshold)
        faces_list = list(face_indices)

        # 计算每个面的法线
        face_normals: Dict[int, Tuple[float, float, float]] = {}
        for fi in face_indices:
            fv1, fv2, fv3 = raw_faces[fi][0], raw_faces[fi][1], raw_faces[fi][2]
            face_normals[fi] = self._calc_face_normal(vertices, fv1, fv2, fv3)

        # 构建边-面邻接
        edge_faces: Dict[Tuple[int, int], List[int]] = defaultdict(list)
        for fi in face_indices:
            fv1, fv2, fv3 = raw_faces[fi][0], raw_faces[fi][1], raw_faces[fi][2]
            for edge in [(min(fv1, fv2), max(fv1, fv2)),
                         (min(fv2, fv3), max(fv2, fv3)),
                         (min(fv3, fv1), max(fv3, fv1))]:
                edge_faces[edge].append(fi)

        # 检测锐边
        sharp_edges: Set[Tuple[int, int]] = set()
        for edge, adj in edge_faces.items():
            if len(adj) == 2:
                n1 = face_normals[adj[0]]
                n2 = face_normals[adj[1]]
                dot = max(-1.0, min(1.0, n1[0]*n2[0] + n1[1]*n2[1] + n1[2]*n2[2]))
                if math.acos(dot) > threshold_rad:
                    sharp_edges.add(edge)

        # 洪水填充平滑组
        face_group: Dict[int, int] = {}
        curr_group = 0

        def flood(start: int, gid: int) -> None:
            stack = [start]
            face_group[start] = gid
            while stack:
                fi = stack.pop()
                fv1, fv2, fv3 = raw_faces[fi][0], raw_faces[fi][1], raw_faces[fi][2]
                for edge in [(min(fv1, fv2), max(fv1, fv2)),
                             (min(fv2, fv3), max(fv2, fv3)),
                             (min(fv3, fv1), max(fv3, fv1))]:
                    if edge not in sharp_edges:
                        for af in edge_faces[edge]:
                            if af not in face_group:
                                face_group[af] = gid
                                stack.append(af)

        for fi in face_indices:
            if fi not in face_group:
                flood(fi, curr_group)
                curr_group += 1

        # 按(顶点, 平滑组)累加法线
        vg_acc: Dict[Tuple[int, int], List[float]] = defaultdict(lambda: [0.0, 0.0, 0.0])
        for fi in face_indices:
            gid = face_group[fi]
            fn = face_normals[fi]
            for v in (raw_faces[fi][0], raw_faces[fi][1], raw_faces[fi][2]):
                key = (v, gid)
                vg_acc[key][0] += fn[0]
                vg_acc[key][1] += fn[1]
                vg_acc[key][2] += fn[2]

        # 归一化
        ni_map: Dict[Tuple[int, int], int] = {}
        for key, acc in vg_acc.items():
            length = math.sqrt(acc[0]**2 + acc[1]**2 + acc[2]**2)
            if length > EPSILON:
                fn = (acc[0]/length, acc[1]/length, acc[2]/length)
            else:
                fn = (0.0, 1.0, 0.0)
            ni_map[key] = len(normals)
            normals.append(fn)

        # 返回顶点->法线映射（取各组中第一个）
        result: Dict[int, int] = {}
        for (v, gid), ni in ni_map.items():
            if v not in result:
                result[v] = ni

        return result

    # ---- 按预计算 smooth_groups 做法线（零件决定硬边） ----

    def _normals_by_groups(
        self,
        vertices: List[Tuple[float, float, float]],
        raw_faces: List[Tuple],
        face_indices: Set[int],
        smooth_groups: Dict[int, int],
        normals: List[Tuple[float, float, float]],
        invert: bool = False,
    ) -> Dict[Tuple[int, int], int]:
        """使用预计算的 smooth_groups 计算顶点法线。

        同一 (顶点, group_id) 内平均面法线，不同 group 间法线拆分（硬边）。

        Returns:
            顶点索引 -> 法线索引的映射。
        """
        # (v_idx, group_id) -> 累加法线
        vg_acc: Dict[Tuple[int, int], List[float]] = defaultdict(lambda: [0.0, 0.0, 0.0])

        for fi in face_indices:
            gid = smooth_groups.get(fi, 0)
            fn = self._calc_face_normal(vertices,
                                        raw_faces[fi][0],
                                        raw_faces[fi][1],
                                        raw_faces[fi][2])
            if invert:
                fn = (-fn[0], -fn[1], -fn[2])
            for corner in range(3):
                v = raw_faces[fi][corner]
                key = (v, gid)
                vg_acc[key][0] += fn[0]
                vg_acc[key][1] += fn[1]
                vg_acc[key][2] += fn[2]

        # 归一化：以 (v, gid) 为键保留硬边多法线
        result: Dict[Tuple[int, int], int] = {}
        for (v, gid), acc in vg_acc.items():
            length = math.sqrt(acc[0]**2 + acc[1]**2 + acc[2]**2)
            if length > EPSILON:
                fn = (acc[0]/length, acc[1]/length, acc[2]/length)
            else:
                fn = (0.0, 1.0, 0.0)
            result[(v, gid)] = len(normals)
            normals.append(fn)

        return result

    # ---- 平滑组法线（鼻锥用，无预计算 smooth_groups 时的 fallback） ----

    def _smooth_group_normals(
        self,
        vertices: List[Tuple[float, float, float]],
        raw_faces: List[Tuple],
        face_indices: Set[int],
        normals: List[Tuple[float, float, float]],
        invert: bool = False,
    ) -> Dict[int, int]:
        """为指定面集合计算顶点平滑法线（平均相邻面法线）。"""
        vf: Dict[int, List[Tuple[float, float, float]]] = defaultdict(list)

        for fi in face_indices:
            fv1, fv2, fv3 = raw_faces[fi][0], raw_faces[fi][1], raw_faces[fi][2]
            fn = self._calc_face_normal(vertices, fv1, fv2, fv3)
            if invert:
                fn = (-fn[0], -fn[1], -fn[2])
            for v in (fv1, fv2, fv3):
                vf[v].append(fn)

        result: Dict[int, int] = {}
        for v, fn_list in vf.items():
            if not fn_list:
                continue
            ax = sum(n[0] for n in fn_list) / len(fn_list)
            ay = sum(n[1] for n in fn_list) / len(fn_list)
            az = sum(n[2] for n in fn_list) / len(fn_list)
            length = math.sqrt(ax**2 + ay**2 + az**2)
            if length > EPSILON:
                fn = (ax/length, ay/length, az/length)
            else:
                fn = (0.0, 1.0, 0.0)
            result[v] = len(normals)
            normals.append(fn)

        return result

    # ---- 简单法线（回退） ----

    def _compute_simple(self, geom: RawGeometry) -> List[Tuple[float, float, float]]:
        """为每个面计算独立的简单面法线。"""
        normals: List[Tuple[float, float, float]] = []
        faces_out: list = []

        for face in geom.faces:
            v1, v2, v3, vt1, vt2, vt3, mat, pid = face
            vn = self._add_face_normal(geom.vertices, v1, v2, v3, normals)
            faces_out.append((v1, v2, v3, vt1, vt2, vt3, vn, vn, vn, mat, pid))

        geom.faces = faces_out
        return normals

    # ---- 基础工具 ----

    @staticmethod
    def _calc_face_normal(
        vertices: List[Tuple[float, float, float]], v1: int, v2: int, v3: int
    ) -> Tuple[float, float, float]:
        """计算单个面的单位法线向量。"""
        nv = len(vertices)
        if v1 >= nv or v2 >= nv or v3 >= nv:
            return (0.0, 1.0, 0.0)

        p1, p2, p3 = vertices[v1], vertices[v2], vertices[v3]
        e1 = (p2[0]-p1[0], p2[1]-p1[1], p2[2]-p1[2])
        e2 = (p3[0]-p1[0], p3[1]-p1[1], p3[2]-p1[2])

        nx = e1[1]*e2[2] - e1[2]*e2[1]
        ny = e1[2]*e2[0] - e1[0]*e2[2]
        nz = e1[0]*e2[1] - e1[1]*e2[0]

        length = math.sqrt(nx**2 + ny**2 + nz**2)
        if length > EPSILON:
            return (nx/length, ny/length, nz/length)
        return (0.0, 1.0, 0.0)

    @staticmethod
    def _add_face_normal(
        vertices: List[Tuple[float, float, float]],
        v1: int, v2: int, v3: int,
        normals: List[Tuple[float, float, float]],
    ) -> int:
        """计算面法线并添加到法线列表。返回法线索引。"""
        fn = NormalService._calc_face_normal(vertices, v1, v2, v3)
        normals.append(fn)
        return len(normals) - 1
