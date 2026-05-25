#!/usr/bin/env python3
"""算法级常量。零件尺寸默认值不放这里，放在各零件模块内。"""

from __future__ import annotations

# 浮点比较阈值
EPSILON: float = 1e-6

# 硬边拆分角度阈值（度），相邻面法线夹角超过此值视为硬边
HARD_EDGE_ANGLE_DEG: float = 40.0

# 端盖法线平滑判定角度阈值（基于 corner_radius 自适应选择）
CAP_SMOOTH_HIGH_CORNER_DEG: float = 135.0   # corner > 0.9（接近圆形）
CAP_SMOOTH_DEFAULT_DEG: float = 150.0       # 默认
CAP_SMOOTH_LOW_CORNER_DEG: float = 170.0    # corner < 0.1（接近方形）
