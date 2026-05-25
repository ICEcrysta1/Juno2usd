#!/usr/bin/env python3
"""MaterialService: 材质/颜色绑定服务。管理材质定义并提供绑定。"""

from __future__ import annotations

from typing import Dict, List

from ..models import Material


class MaterialService:
    """材质服务，管理材质定义和查找。"""

    def __init__(self) -> None:
        self._materials: Dict[str, Material] = {}

    def load_materials(self, materials: List[Material]) -> None:
        """加载材质列表。

        Args:
            materials: Material 对象列表。
        """
        self._materials = {m.name: m for m in materials}

    def get(self, name: str) -> Material:
        """根据名称获取材质。找不到返回默认灰色材质。

        Args:
            name: 材质名称。

        Returns:
            Material 对象。
        """
        if name in self._materials:
            return self._materials[name]
        return Material(name=name, color=(0.5, 0.5, 0.5), metallic=0.0, roughness=0.5)

    @property
    def materials(self) -> Dict[str, Material]:
        return self._materials
