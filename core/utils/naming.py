#!/usr/bin/env python3
"""命名工具函数。"""

from __future__ import annotations


def clean_material_name(name: str) -> str:
    """清理材质名称以符合 USD 命名规范。

    将空格、横线、点号、斜杠替换为下划线。

    Args:
        name: 原始材质名称。

    Returns:
        清理后的名称。
    """
    return name.replace(" ", "_").replace("-", "_").replace(".", "_").replace("/", "_")
