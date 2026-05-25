#!/usr/bin/env python3
"""纯数学工具函数：旋转矩阵、圆角矩形算法、XML 解析辅助。"""

from __future__ import annotations

import math
from typing import List, Tuple

import numpy as np

from .constants import EPSILON


def parse_vector(text: str) -> Tuple[float, float, float]:
    """解析逗号分隔的向量字符串 "x,y,z"。

    Args:
        text: 逗号分隔的数值字符串。

    Returns:
        三元组 (x, y, z)。

    Raises:
        ValueError: 输入格式不正确（不是恰好 3 个值）。
    """
    parts = text.split(",")
    if len(parts) != 3:
        raise ValueError(f"向量需要恰好 3 个逗号分隔的值，实际得到 {len(parts)}: {text!r}")
    return (float(parts[0]), float(parts[1]), float(parts[2]))


def parse_scale(text: str) -> Tuple[float, float]:
    """解析 Fuselage 的 scale 参数 "x,z"。

    Args:
        text: 逗号分隔的数值字符串。

    Returns:
        二元组 (x, z)。

    Raises:
        ValueError: 输入格式不正确（不是恰好 2 个值）。
    """
    parts = text.split(",")
    if len(parts) != 2:
        raise ValueError(f"scale 需要恰好 2 个逗号分隔的值，实际得到 {len(parts)}: {text!r}")
    return (float(parts[0]), float(parts[1]))


def rotation_matrix(euler_angles: Tuple[float, float, float]) -> np.ndarray:
    """从 Unity 欧拉角（度数）创建旋转矩阵。

    Unity 使用 ZXY 旋转顺序：先 Z，再 X，再 Y。

    Args:
        euler_angles: (rx, ry, rz) 度数欧拉角。

    Returns:
        3x3 NumPy 旋转矩阵。
    """
    rx, ry, rz = (math.radians(a) for a in euler_angles)

    cz, sz = math.cos(rz), math.sin(rz)
    cx, sx = math.cos(rx), math.sin(rx)
    cy, sy = math.cos(ry), math.sin(ry)

    Rz = np.array([
        [cz, -sz, 0],
        [sz, cz, 0],
        [0, 0, 1],
    ])
    Rx = np.array([
        [1, 0, 0],
        [0, cx, -sx],
        [0, sx, cx],
    ])
    Ry = np.array([
        [cy, 0, sy],
        [0, 1, 0],
        [-sy, 0, cy],
    ])

    # ZXY 顺序: Ry * Rx * Rz
    return Ry @ Rx @ Rz


def get_rounded_rect_point(
    angle: float,
    radius_x: float,
    radius_z: float,
    corner_radii: Tuple[float, ...],
    deformation: float = 0.0,
) -> Tuple[float, float]:
    """生成圆角矩形截面上的点坐标。

    算法：
    1. 计算标准圆角矩形/圆/方形上的点
    2. 应用泪滴变形（根据 Z 收缩 X）
    3. 应用 radius_x/radius_z 缩放

    Args:
        angle: 当前角度（弧度）。
        radius_x: X 轴半径。
        radius_z: Z 轴半径。
        corner_radii: 四角圆角半径，至少 4 个值。
        deformation: 泪滴变形量，0=正常，0.5=收缩，1=底部收缩到点。

    Returns:
        (x, z) 坐标。
    """
    normalized_angle = angle % (2 * math.pi)

    # 确定象限
    if math.pi / 2 <= normalized_angle < math.pi:
        corner_idx = 0  # top-left
        sign_x, sign_z = -1, 1
    elif 0 <= normalized_angle < math.pi / 2:
        corner_idx = 1  # top-right
        sign_x, sign_z = 1, 1
    elif 3 * math.pi / 2 <= normalized_angle < 2 * math.pi:
        corner_idx = 2  # bottom-right
        sign_x, sign_z = 1, -1
    else:
        corner_idx = 3  # bottom-left
        sign_x, sign_z = -1, -1

    corner_radius = corner_radii[corner_idx] if corner_idx < len(corner_radii) else 1.0

    # Step 1: 计算标准圆角矩形点
    if corner_radius <= 0.001:
        # 纯方形
        abs_cos = abs(math.cos(normalized_angle))
        abs_sin = abs(math.sin(normalized_angle))
        if abs_cos < 0.0001 or abs_sin < 0.0001:
            t = 1.0
        else:
            t = min(1.0 / abs_cos, 1.0 / abs_sin)
        unit_x = t * math.cos(normalized_angle)
        unit_z = t * math.sin(normalized_angle)
    elif corner_radius >= 0.999:
        # 纯圆形
        unit_x = math.cos(normalized_angle)
        unit_z = math.sin(normalized_angle)
    else:
        # 圆角矩形
        r = corner_radius
        cx = sign_x * (1 - r)
        cz = sign_z * (1 - r)

        angle_corner = math.atan2(sign_z, sign_x) % (2 * math.pi)
        angle_diff = abs(normalized_angle - angle_corner)
        if angle_diff > math.pi:
            angle_diff = 2 * math.pi - angle_diff

        # 切线点角度差
        tangent1_x, tangent1_z = sign_x, sign_z * (1 - r)
        angle_tangent1 = math.atan2(tangent1_z, tangent1_x) % (2 * math.pi)
        diff_tangent = abs(angle_tangent1 - angle_corner)
        if diff_tangent > math.pi:
            diff_tangent = 2 * math.pi - diff_tangent

        if angle_diff <= diff_tangent:
            # 在圆弧上
            unit_x = cx + r * math.cos(normalized_angle)
            unit_z = cz + r * math.sin(normalized_angle)
        else:
            # 在直边上
            abs_cos = abs(math.cos(normalized_angle))
            abs_sin = abs(math.sin(normalized_angle))
            if abs_cos < 0.0001 or abs_sin < 0.0001:
                t = 1.0
            else:
                t = min(1.0 / abs_cos, 1.0 / abs_sin)
            unit_x = t * math.cos(normalized_angle)
            unit_z = t * math.sin(normalized_angle)

    # Step 2: 泪滴变形（根据 Z 收缩 X）
    if deformation > 0.001:
        z_norm = max(-1.0, min(1.0, unit_z))
        width_ratio = (z_norm + 1.0) / 2.0
        scale = 1.0 - deformation * (1.0 - width_ratio)
        unit_x = unit_x * scale

    # Step 3: 应用缩放
    x = radius_x * unit_x
    z = radius_z * unit_z

    return x, z


def apply_clamp(
    coords: List[Tuple[float, float, float]],
    clamp_vals: Tuple[float, ...],
) -> List[Tuple[float, float, float]]:
    """对坐标列表应用层级挤压效果。

    Args:
        coords: 坐标列表 [(x, y, z), ...]。
        clamp_vals: 挤压参数 (tx_neg, tx_pos, tz_neg, tz_pos)，值为 0=挤压到中心，1=不挤压。

    Returns:
        挤压后的坐标列表。
    """
    if len(clamp_vals) < 4:
        return coords

    max_ratio_x_neg = abs(clamp_vals[0])
    max_ratio_x_pos = abs(clamp_vals[1])
    max_ratio_z_neg = abs(clamp_vals[2])
    max_ratio_z_pos = abs(clamp_vals[3])

    # 都不挤压，直接返回
    if (
        max_ratio_x_neg >= 0.999
        and max_ratio_x_pos >= 0.999
        and max_ratio_z_neg >= 0.999
        and max_ratio_z_pos >= 0.999
    ):
        return coords

    new_coords: List[Tuple[float, float, float]] = []
    for x, y, z in coords:
        new_x = x
        new_z = z

        if x < 0 and max_ratio_x_neg < 0.999:
            new_x = x * max_ratio_x_neg
        elif x > 0 and max_ratio_x_pos < 0.999:
            new_x = x * max_ratio_x_pos

        if z < 0 and max_ratio_z_neg < 0.999:
            new_z = z * max_ratio_z_neg
        elif z > 0 and max_ratio_z_pos < 0.999:
            new_z = z * max_ratio_z_pos

        new_coords.append((new_x, y, new_z))

    return new_coords
