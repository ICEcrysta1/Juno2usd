#!/usr/bin/env python3
"""零件模块抽象基类。每个零件只管自己的几何生成和父级查找。"""

from __future__ import annotations

from abc import ABC, abstractmethod
from typing import Optional

from ..models import FuselageParams, RawGeometry


class PartBase(ABC):
    """零件模块基类。

    每个具体零件类型实现:
    - generate(): 生成裸几何（顶点 + 面索引 + ring_info）
    - find_parent(): 在连接图中查找父级零件
    """

    @abstractmethod
    def generate(self, params: FuselageParams, segments: int) -> RawGeometry:
        """生成纯几何数据。

        Args:
            params: Fuselage 几何参数（长度、半径、变形等）。
            segments: 圆周细分数。

        Returns:
            RawGeometry: 包含顶点、面索引、ring_info 的原始几何。
        """
        ...

    @abstractmethod
    def find_parent(self, part_id: str, connections: dict[str, Optional[str]]) -> Optional[str]:
        """在连接图中查找自己的父级零件 ID。

        为骨骼系统预留，当前流水线不使用返回值。

        Args:
            part_id: 当前零件 ID。
            connections: child_to_parent 映射。

        Returns:
            父级零件 ID，或 None。
        """
        ...
