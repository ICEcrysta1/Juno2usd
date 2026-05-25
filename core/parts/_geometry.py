#!/usr/bin/env python3
"""零件几何生成算法（私有模块，供各零件模块调用）。"""

from __future__ import annotations

import math
from typing import List, Optional, Tuple

import numpy as np

from ..models import FuselageParams, RawGeometry
from ..utils.constants import EPSILON
from ..utils.math3d import apply_clamp, get_rounded_rect_point, rotation_matrix


def generate_cylinder_geometry(
    part_id: str,
    material_name: str,
    part_type: str,
    params: FuselageParams,
    segments: int,
    *,
    is_hollow: bool = False,
    wall_thickness: float = 0.0,
) -> RawGeometry:
    """生成圆柱体几何（实心或空心）。

    Args:
        part_id: 零件 ID。
        material_name: 材质名称。
        part_type: 零件类型字符串。
        params: Fuselage 几何参数。
        segments: 圆周细分数。
        is_hollow: 是否空心（Inlet/Fairing 类）。
        wall_thickness: 壁厚（空心零件用）。
    """
    geom = RawGeometry(part_id=part_id, material_name=material_name)
    R = rotation_matrix(params.deformations)  # 会被覆盖，先占位
    del R  # 不在这里做变换

    half_len = params.length / 2
    scale_x, scale_y, scale_z = params.part_scale
    has_part_scale = (
        abs(scale_x - 1.0) > EPSILON
        or abs(scale_y - 1.0) > EPSILON
        or abs(scale_z - 1.0) > EPSILON
    )
    angle_step = 2 * math.pi / segments

    top_corners = params.top_corners
    bottom_corners = params.bottom_corners
    top_def = params.top_deformation
    vertical_shear = params.vertical_shear
    bottom_def = params.bottom_deformation
    top_clamp = params.top_clamp
    bottom_clamp = params.bottom_clamp

    bottom_is_point = params.bottom_scale_x < EPSILON and params.bottom_scale_z < EPSILON
    top_is_point = params.top_scale_x < EPSILON and params.top_scale_z < EPSILON

    # 生成原始截面坐标
    raw_bottom: List[Tuple[float, float, float]] = []
    raw_top: List[Tuple[float, float, float]] = []
    raw_bottom_inner: Optional[List[Tuple[float, float, float]]] = [] if is_hollow else None
    raw_top_inner: Optional[List[Tuple[float, float, float]]] = [] if is_hollow else None

    for i in range(segments):
        angle = i * angle_step

        # 底部
        if bottom_is_point:
            bx, bz = -params.offset_x, -params.offset_z
        else:
            bx, bz = get_rounded_rect_point(
                angle, params.radius_x, params.radius_z, bottom_corners, bottom_def
            )
            bx = bx * params.bottom_scale_x - params.offset_x
            bz = bz * params.bottom_scale_z - params.offset_z
        raw_bottom.append((bx, -half_len, bz))

        if is_hollow and raw_bottom_inner is not None:
            if bottom_is_point:
                raw_bottom_inner.append((0.0, -half_len, 0.0))
            else:
                dist = math.sqrt(bx**2 + bz**2)
                if dist > wall_thickness:
                    ratio = (dist - wall_thickness) / dist
                    raw_bottom_inner.append((bx * ratio, -half_len, bz * ratio))
                else:
                    raw_bottom_inner.append((0.0, -half_len, 0.0))

        # 顶部
        if top_is_point:
            tx, tz = params.offset_x, params.offset_z
        else:
            tx, tz = get_rounded_rect_point(
                angle, params.radius_x, params.radius_z, top_corners, top_def
            )
            tx = tx * params.top_scale_x + params.offset_x
            tz = tz * params.top_scale_z + params.offset_z
        raw_top.append((tx, half_len, tz))

        if is_hollow and raw_top_inner is not None:
            if top_is_point:
                raw_top_inner.append((0.0, half_len, 0.0))
            else:
                dist = math.sqrt(tx**2 + tz**2)
                if dist > wall_thickness:
                    ratio = (dist - wall_thickness) / dist
                    raw_top_inner.append((tx * ratio, half_len, tz * ratio))
                else:
                    raw_top_inner.append((0.0, half_len, 0.0))

    # Clamp + partScale
    squeezed_bottom = apply_clamp(raw_bottom, bottom_clamp)
    squeezed_top = apply_clamp(raw_top, top_clamp)

    if is_hollow and raw_bottom_inner is not None and raw_top_inner is not None:
        squeezed_bottom_inner = apply_clamp(raw_bottom_inner, bottom_clamp)
        squeezed_top_inner = apply_clamp(raw_top_inner, top_clamp)
    else:
        squeezed_bottom_inner = None
        squeezed_top_inner = None

    if has_part_scale:
        squeezed_bottom = [(x*scale_x, y*scale_y, z*scale_z) for x, y, z in squeezed_bottom]
        squeezed_top = [(x*scale_x, y*scale_y, z*scale_z) for x, y, z in squeezed_top]
        if squeezed_bottom_inner is not None:
            squeezed_bottom_inner = [(x*scale_x, y*scale_y, z*scale_z) for x, y, z in squeezed_bottom_inner]
        if squeezed_top_inner is not None:
            squeezed_top_inner = [(x*scale_x, y*scale_y, z*scale_z) for x, y, z in squeezed_top_inner]

    # 生成顶点和面（不做旋转/平移，不做最终法线）
    bottom_indices: List[Tuple[int, int]] = []
    top_indices: List[Tuple[int, int]] = []
    bottom_inner_indices: Optional[List[Tuple[int, int]]] = [] if is_hollow else None
    top_inner_indices: Optional[List[Tuple[int, int]]] = [] if is_hollow else None

    # 底面中心点（如果是点状）
    v_center_bottom = -1
    vt_center_bottom = -1
    if bottom_is_point:
        bx, by, bz = squeezed_bottom[0]
        v_center_bottom = len(geom.vertices)
        geom.vertices.append((bx, by, bz))
        vt_center_bottom = len(geom.uvs)
        geom.uvs.append((0.5, 0.5))

    v_center_top = -1
    vt_center_top = -1
    if top_is_point:
        tx, ty, tz = squeezed_top[0]
        z_norm = tz / (params.radius_z * params.top_scale_z) if params.radius_z * params.top_scale_z > 0.001 else 0.0
        cut = 2 * params.offset_y * vertical_shear
        ty += cut * (z_norm - 1) / 2
        v_center_top = len(geom.vertices)
        geom.vertices.append((tx, ty, tz))
        vt_center_top = len(geom.uvs)
        geom.uvs.append((0.5, 0.5))

    for i in range(segments):
        u = i / segments

        # 底部外顶点
        if bottom_is_point:
            bottom_indices.append((v_center_bottom, vt_center_bottom))
        else:
            bx, by, bz = squeezed_bottom[i]
            vi = len(geom.vertices)
            geom.vertices.append((bx, by, bz))
            vti = len(geom.uvs)
            geom.uvs.append((u, 0.0))
            bottom_indices.append((vi, vti))

        # 顶部外顶点
        if top_is_point:
            top_indices.append((v_center_top, vt_center_top))
        else:
            tx, ty, tz = squeezed_top[i]
            z_norm = tz / (params.radius_z * params.top_scale_z) if params.radius_z * params.top_scale_z > 0.001 else 0.0
            cut = 2 * params.offset_y * vertical_shear
            ty += cut * (z_norm - 1) / 2
            vi = len(geom.vertices)
            geom.vertices.append((tx, ty, tz))
            vti = len(geom.uvs)
            geom.uvs.append((u, 1.0))
            top_indices.append((vi, vti))

        # 内层顶点（空心）
        if is_hollow and bottom_inner_indices is not None and top_inner_indices is not None:
            if bottom_is_point:
                bottom_inner_indices.append((v_center_bottom, vt_center_bottom))
            else:
                bxi, byi, bzi = squeezed_bottom_inner[i]  # type: ignore[index]
                vi = len(geom.vertices)
                geom.vertices.append((bxi, byi, bzi))
                vti = len(geom.uvs)
                geom.uvs.append((u, 0.0))
                bottom_inner_indices.append((vi, vti))

            if top_is_point:
                top_inner_indices.append((v_center_top, vt_center_top))
            else:
                txi, tyi, tzi = squeezed_top_inner[i]  # type: ignore[index]
                z_norm_i = tzi / (params.radius_z * params.top_scale_z) if params.radius_z * params.top_scale_z > 0.001 else 0.0
                cut_i = 2 * params.offset_y * vertical_shear
                tyi += cut_i * (z_norm_i - 1) / 2
                vi = len(geom.vertices)
                geom.vertices.append((txi, tyi, tzi))
                vti = len(geom.uvs)
                geom.uvs.append((u, 1.0))
                top_inner_indices.append((vi, vti))

    # 生成面索引
    _add_cylinder_faces(
        geom, segments, bottom_indices, top_indices,
        bottom_inner_indices, top_inner_indices,
        bottom_is_point, top_is_point, is_hollow,
        part_type, params, has_part_scale, scale_x, scale_y, scale_z,
        vertical_shear,
    )

    # ring_info
    geom.ring_info = {
        "bottom_indices": [idx for idx, _ in bottom_indices],
        "top_indices": [idx for idx, _ in top_indices],
        "bottom_inner_indices": [idx for idx, _ in bottom_inner_indices] if is_hollow and bottom_inner_indices else None,
        "top_inner_indices": [idx for idx, _ in top_inner_indices] if is_hollow and top_inner_indices else None,
        "bottom_corners": bottom_corners,
        "top_corners": top_corners,
        "corner_radiuses": params.corner_radiuses,
        "is_inlet": is_hollow,
        "part_type": part_type,
        "segments": segments,
        "bottom_is_point": bottom_is_point,
        "top_is_point": top_is_point,
    }

    return geom


def _add_cylinder_faces(
    geom: RawGeometry,
    segments: int,
    bottom_indices: List[Tuple[int, int]],
    top_indices: List[Tuple[int, int]],
    bottom_inner_indices: Optional[List[Tuple[int, int]]],
    top_inner_indices: Optional[List[Tuple[int, int]]],
    bottom_is_point: bool,
    top_is_point: bool,
    is_hollow: bool,
    part_type: str,
    params: FuselageParams,
    has_part_scale: bool,
    scale_x: float,
    scale_y: float,
    scale_z: float,
    vertical_shear: float,
) -> None:
    """为圆柱体添加面索引。"""
    # 外侧面
    if bottom_is_point and top_is_point:
        pass
    elif bottom_is_point:
        for i in range(segments):
            ni = (i + 1) % segments
            b_i = bottom_indices[i]
            t_i = top_indices[i]
            t_n = top_indices[ni]
            geom.faces.append((b_i[0], t_i[0], t_n[0], b_i[1], t_i[1], t_n[1], geom.material_name, part_type))
    elif top_is_point:
        for i in range(segments):
            ni = (i + 1) % segments
            b_i = bottom_indices[i]
            b_n = bottom_indices[ni]
            t_i = top_indices[i]
            geom.faces.append((b_i[0], t_i[0], b_n[0], b_i[1], t_i[1], b_n[1], geom.material_name, part_type))
    else:
        for i in range(segments):
            ni = (i + 1) % segments
            b_i = bottom_indices[i]
            b_n = bottom_indices[ni]
            t_n = top_indices[ni]
            t_i = top_indices[i]
            geom.faces.append((b_i[0], t_i[0], t_n[0], b_i[1], t_i[1], t_n[1], geom.material_name, part_type))
            geom.faces.append((b_i[0], t_n[0], b_n[0], b_i[1], t_n[1], b_n[1], geom.material_name, part_type))

    if is_hollow and bottom_inner_indices is not None and top_inner_indices is not None:
        # 内侧面（法线反向）
        if bottom_is_point and top_is_point:
            pass
        elif bottom_is_point:
            for i in range(segments):
                ni = (i + 1) % segments
                bi_i = bottom_inner_indices[i]
                ti_i = top_inner_indices[i]
                ti_n = top_inner_indices[ni]
                geom.faces.append((bi_i[0], ti_n[0], ti_i[0], bi_i[1], ti_n[1], ti_i[1], geom.material_name, part_type))
        elif top_is_point:
            for i in range(segments):
                ni = (i + 1) % segments
                bi_i = bottom_inner_indices[i]
                bi_n = bottom_inner_indices[ni]
                ti_i = top_inner_indices[i]
                geom.faces.append((bi_i[0], bi_n[0], ti_i[0], bi_i[1], bi_n[1], ti_i[1], geom.material_name, part_type))
        else:
            for i in range(segments):
                ni = (i + 1) % segments
                bi_i = bottom_inner_indices[i]
                bi_n = bottom_inner_indices[ni]
                ti_n = top_inner_indices[ni]
                ti_i = top_inner_indices[i]
                geom.faces.append((bi_i[0], ti_n[0], ti_i[0], bi_i[1], ti_n[1], ti_i[1], geom.material_name, part_type))
                geom.faces.append((bi_i[0], bi_n[0], ti_n[0], bi_i[1], bi_n[1], ti_n[1], geom.material_name, part_type))

        # 端盖环（空心）
        if not bottom_is_point:
            for i in range(segments):
                ni = (i + 1) % segments
                bo_i = bottom_indices[i]
                bo_n = bottom_indices[ni]
                bi_n = bottom_inner_indices[ni]
                bi_i = bottom_inner_indices[i]
                geom.faces.append((bo_i[0], bo_n[0], bi_n[0], bo_i[1], bo_n[1], bi_n[1], geom.material_name, part_type))
                geom.faces.append((bo_i[0], bi_n[0], bi_i[0], bo_i[1], bi_n[1], bi_i[1], geom.material_name, part_type))
        if not top_is_point:
            for i in range(segments):
                ni = (i + 1) % segments
                to_i = top_indices[i]
                to_n = top_indices[ni]
                ti_n = top_inner_indices[ni]
                ti_i = top_inner_indices[i]
                geom.faces.append((to_i[0], ti_i[0], ti_n[0], to_i[1], ti_i[1], ti_n[1], geom.material_name, part_type))
                geom.faces.append((to_i[0], ti_n[0], to_n[0], to_i[1], ti_n[1], to_n[1], geom.material_name, part_type))
    else:
        # 实心端盖
        if not bottom_is_point:
            center_local = (-params.offset_x * scale_x, -params.length / 2 * scale_y, -params.offset_z * scale_z)
            v_cb = len(geom.vertices)
            geom.vertices.append(center_local)
            vt_cb = len(geom.uvs)
            geom.uvs.append((0.5, 0.5))
            for i in range(segments):
                ni = (i + 1) % segments
                b_i = bottom_indices[i]
                b_n = bottom_indices[ni]
                geom.faces.append((v_cb, b_i[0], b_n[0], vt_cb, b_i[1], b_n[1], geom.material_name, part_type))

        if not top_is_point:
            cut = 2 * params.offset_y * vertical_shear * scale_y
            center_local = (params.offset_x * scale_x, params.length / 2 * scale_y - cut/2, params.offset_z * scale_z)
            v_ct = len(geom.vertices)
            geom.vertices.append(center_local)
            vt_ct = len(geom.uvs)
            geom.uvs.append((0.5, 0.5))
            for i in range(segments):
                ni = (i + 1) % segments
                t_i = top_indices[i]
                t_n = top_indices[ni]
                geom.faces.append((v_ct, t_n[0], t_i[0], vt_ct, t_n[1], t_i[1], geom.material_name, part_type))


def generate_cone_geometry(
    part_id: str,
    material_name: str,
    part_type: str,
    params: FuselageParams,
    segments: int,
    *,
    is_hollow: bool = False,
    wall_thickness: float = 0.0,
    subdivisions: int = 5,
) -> RawGeometry:
    """生成鼻锥几何（实心或空心）。

    Args:
        part_id: 零件 ID。
        material_name: 材质名称。
        part_type: 零件类型字符串。
        params: Fuselage 几何参数。
        segments: 圆周细分数。
        is_hollow: 是否空心。
        wall_thickness: 壁厚（空心零件用）。
        subdivisions: 纵向细分数。
    """
    geom = RawGeometry(part_id=part_id, material_name=material_name)
    half_len = params.length / 2
    scale_x, scale_y, scale_z = params.part_scale
    has_part_scale = (
        abs(scale_x - 1.0) > EPSILON
        or abs(scale_y - 1.0) > EPSILON
        or abs(scale_z - 1.0) > EPSILON
    )
    angle_step = 2 * math.pi / segments
    num_rings = subdivisions + 1

    top_corners = params.top_corners
    bottom_corners = params.bottom_corners
    top_def = params.top_deformation
    vertical_shear = params.vertical_shear
    bottom_def = params.bottom_deformation
    top_clamp = params.top_clamp
    bottom_clamp = params.bottom_clamp

    ring_indices: List[List[Tuple[int, int]]] = []
    inner_ring_indices: Optional[List[List[Tuple[int, int]]]] = [] if is_hollow else None

    for ring in range(num_rings):
        t = ring / subdivisions
        y = -half_len + t * params.length

        if t < 0.99:
            ease_t = math.pow(t, 2.1)
            r_ratio_x = params.bottom_scale_x * (1.0 - ease_t) + params.top_scale_x * ease_t
            r_ratio_z = params.bottom_scale_z * (1.0 - ease_t) + params.top_scale_z * ease_t
        else:
            r_ratio_x = 0.0
            r_ratio_z = 0.0

        current_offset_x = params.offset_x * t
        current_offset_z = params.offset_z * t
        current_def = bottom_def * (1.0 - t) + top_def * t

        current_corners = tuple(
            bottom_corners[i] * (1.0 - t) + top_corners[i] * t
            for i in range(4)
        )
        current_clamp = tuple(
            bottom_clamp[i] * (1.0 - t) + top_clamp[i] * t
            for i in range(4)
        )

        current_rx = params.radius_x * r_ratio_x
        current_rz = params.radius_z * r_ratio_z

        raw_coords: List[Tuple[float, float, float]] = []
        for i in range(segments):
            angle = i * angle_step
            x, z = get_rounded_rect_point(angle, params.radius_x, params.radius_z, current_corners, current_def)
            x = x * r_ratio_x + current_offset_x
            z = z * r_ratio_z + current_offset_z
            raw_coords.append((x, y, z))

        squeezed = apply_clamp(raw_coords, current_clamp)
        if has_part_scale:
            squeezed = [(x*scale_x, y*scale_y, z*scale_z) for x, y, z in squeezed]

        is_tip = r_ratio_x < EPSILON and r_ratio_z < EPSILON

        ring_verts: List[Tuple[int, int]] = []
        inner_ring_verts: Optional[List[Tuple[int, int]]] = [] if is_hollow else None

        if is_tip:
            x, y_local, z = squeezed[0]
            z_n = z / (current_rz * scale_z) if current_rz * scale_z > 0.001 else 0.0
            cut = 2 * params.offset_y * vertical_shear * t
            y_final = y_local + cut * (z_n - 1) / 2
            vi = len(geom.vertices)
            geom.vertices.append((x, y_final, z))
            vti = len(geom.uvs)
            geom.uvs.append((0.5, t))
            for _ in range(segments):
                ring_verts.append((vi, vti))

            if is_hollow and inner_ring_verts is not None:
                # 内层尖端：沿 Y 轴向基座方向偏移壁厚，独立顶点
                inner_y = y_final - wall_thickness * scale_y if t > 0.5 else y_final + wall_thickness * scale_y
                vi_inner = len(geom.vertices)
                geom.vertices.append((x, inner_y, z))
                vti_inner = len(geom.uvs)
                geom.uvs.append((0.5, t))
                for _ in range(segments):
                    inner_ring_verts.append((vi_inner, vti_inner))
        else:
            if is_hollow:
                inner_coords: List[Tuple[float, float, float]] = []
                for i in range(segments):
                    x, y_local, z = squeezed[i]
                    dist = math.sqrt(x**2 + z**2)
                    if dist > wall_thickness:
                        ratio = (dist - wall_thickness) / dist
                        inner_coords.append((x * ratio, y_local, z * ratio))
                    else:
                        inner_coords.append((0.0, y_local, 0.0))

            for i in range(segments):
                x, y_local, z = squeezed[i]
                z_n = z / (current_rz * scale_z) if current_rz * scale_z > 0.001 else 0.0
                cut = 2 * params.offset_y * vertical_shear * t
                y_final = y_local + cut * (z_n - 1) / 2
                u = i / segments

                vi = len(geom.vertices)
                geom.vertices.append((x, y_final, z))
                vti = len(geom.uvs)
                geom.uvs.append((u, t))
                ring_verts.append((vi, vti))

                if is_hollow and inner_ring_verts is not None:
                    xi, yi_local, zi = inner_coords[i]
                    zi_n = zi / (current_rz * scale_z) if current_rz * scale_z > 0.001 else 0.0
                    cut_i = 2 * params.offset_y * vertical_shear * t
                    y_final_i = yi_local + cut_i * (zi_n - 1) / 2
                    vi_i = len(geom.vertices)
                    geom.vertices.append((xi, y_final_i, zi))
                    vti_i = len(geom.uvs)
                    geom.uvs.append((u, t))
                    inner_ring_verts.append((vi_i, vti_i))

        ring_indices.append(ring_verts)
        if is_hollow and inner_ring_indices is not None and inner_ring_verts is not None:
            inner_ring_indices.append(inner_ring_verts)

    # 侧面
    for ring in range(subdivisions):
        curr = ring_indices[ring]
        nxt = ring_indices[ring + 1]
        for i in range(segments):
            ni = (i + 1) % segments
            ci = curr[i]
            cn = curr[ni]
            nn_n = nxt[ni]
            ni_v = nxt[i]
            geom.faces.append((ci[0], ni_v[0], nn_n[0], ci[1], ni_v[1], nn_n[1], geom.material_name, part_type))
            geom.faces.append((ci[0], nn_n[0], cn[0], ci[1], nn_n[1], cn[1], geom.material_name, part_type))

    if is_hollow and inner_ring_indices is not None:
        for ring in range(subdivisions):
            curr = inner_ring_indices[ring]
            nxt = inner_ring_indices[ring + 1]
            for i in range(segments):
                ni = (i + 1) % segments
                ci = curr[i]
                cn = curr[ni]
                nn_n = nxt[ni]
                ni_v = nxt[i]
                geom.faces.append((ci[0], cn[0], nn_n[0], ci[1], cn[1], nn_n[1], geom.material_name, part_type))
                geom.faces.append((ci[0], nn_n[0], ni_v[0], ci[1], nn_n[1], ni_v[1], geom.material_name, part_type))

    # 底部端盖
    has_bottom = params.bottom_scale_x > EPSILON or params.bottom_scale_z > EPSILON
    if has_bottom:
        bottom_ring = ring_indices[0]
        if is_hollow and inner_ring_indices is not None:
            bottom_inner = inner_ring_indices[0]
            for i in range(segments):
                ni = (i + 1) % segments
                bo_i = bottom_ring[i]
                bo_n = bottom_ring[ni]
                bi_n = bottom_inner[ni]
                bi_i = bottom_inner[i]
                geom.faces.append((bo_i[0], bo_n[0], bi_n[0], bo_i[1], bo_n[1], bi_n[1], geom.material_name, part_type))
                geom.faces.append((bo_i[0], bi_n[0], bi_i[0], bo_i[1], bi_n[1], bi_i[1], geom.material_name, part_type))
        else:
            center = (-params.offset_x * scale_x, -half_len * scale_y, -params.offset_z * scale_z)
            vc = len(geom.vertices)
            geom.vertices.append(center)
            vtc = len(geom.uvs)
            geom.uvs.append((0.5, 0.5))
            for i in range(segments):
                ni = (i + 1) % segments
                bi = bottom_ring[i]
                bn = bottom_ring[ni]
                geom.faces.append((vc, bi[0], bn[0], vtc, bi[1], bn[1], geom.material_name, part_type))

    geom.ring_info = {
        "ring_indices": [[idx for idx, _ in ring] for ring in ring_indices],
        "inner_ring_indices": [[idx for idx, _ in ring] for ring in inner_ring_indices] if is_hollow and inner_ring_indices else None,
        "is_nose_cone": True,
        "is_hollow": is_hollow,
        "part_type": part_type,
        "segments": segments,
        "subdivisions": subdivisions,
        "has_bottom_cap": has_bottom,
    }

    return geom
