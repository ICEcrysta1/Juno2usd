#!/usr/bin/env python3
"""入口：SimpleRockets 2 XML → USD 转换。

用法:
    python -m core.main input.xml [output.usda] [--skeleton] [--no-normals] [...]
"""

from __future__ import annotations

import argparse
import os
import sys
from typing import Optional

from .bus import Bus
from .merger import MeshMerger
from .parser import SR2XMLParser
from .utils.naming import clean_material_name


def main() -> None:
    parser = argparse.ArgumentParser(
        description="SimpleRockets 2 到 USD 转换器（重构版）",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
示例:
  python -m core.main input.xml output.usda
  python -m core.main input.xml --radius-x 2.0 --radius-z 1.5
  python -m core.main input.xml --no-normals --keep-cache
  python -m core.main input.xml --skeleton          # 骨骼绑定（开发中）
        """,
    )
    parser.add_argument("input", help="输入 XML 文件路径")
    parser.add_argument("output", nargs="?", help="输出 USD 文件路径（默认 Output/ 目录）")
    parser.add_argument("--radius-x", type=float, default=1.0, help="默认椭圆短边半径（默认: 1.0）")
    parser.add_argument("--radius-z", type=float, default=1.0, help="默认椭圆长边半径（默认: 1.0）")
    parser.add_argument("--segments", type=int, default=24, help="圆柱分段数（默认: 24）")
    parser.add_argument("--no-normals", action="store_true", help="不使用自定义法线")
    parser.add_argument("--keep-cache", action="store_true", help="保留缓存文件")
    parser.add_argument("--prefix", help="网格名称前缀（默认从文件名提取）")
    parser.add_argument("--skeleton", action="store_true", help="启用骨骼绑定（开发中）")

    args = parser.parse_args()

    if args.skeleton:
        print("注意: 骨骼模式暂未在新架构中实现，将以普通模式运行。")

    # 处理输入路径
    xml_file = args.input
    if not os.path.isabs(xml_file):
        project_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        input_dir = os.path.join(project_dir, "Input")
        xml_file = os.path.join(input_dir, xml_file)

    if not os.path.exists(xml_file):
        print(f"错误: 输入文件不存在: {xml_file}")
        sys.exit(1)

    # 处理输出路径
    project_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    output_dir = os.path.join(project_dir, "Output")
    os.makedirs(output_dir, exist_ok=True)

    if args.output:
        if not os.path.isabs(args.output) and not os.path.dirname(args.output):
            output_file = os.path.join(output_dir, args.output)
        else:
            output_file = args.output
    else:
        basename = os.path.splitext(os.path.basename(xml_file))[0]
        suffix = "_skel" if args.skeleton else ""
        output_file = os.path.join(output_dir, f"{basename}{suffix}.usda")

    if args.prefix:
        mesh_prefix = args.prefix
    else:
        mesh_prefix = clean_material_name(os.path.splitext(os.path.basename(xml_file))[0])

    # === 流水线 ===

    print("=" * 60)
    print("步骤 1/3: 解析 XML")
    print("=" * 60)

    xml_parser = SR2XMLParser(
        default_radius_x=args.radius_x,
        default_radius_z=args.radius_z,
    )
    result = xml_parser.parse_file(xml_file)
    parts_list = result["parts"]
    materials_list = result["materials"]

    if not parts_list:
        print("[Main] 错误: 没有找到有效的零件")
        sys.exit(1)

    print(f"[Main] 零件数: {len(parts_list)}, 材质数: {len(materials_list)}")

    print()
    print("=" * 60)
    print("步骤 2/3: 生成网格 + 计算法线")
    print("=" * 60)

    bus = Bus(segments=args.segments)
    use_custom_normals = not args.no_normals
    meshes = bus.assemble(parts_list)

    if not meshes:
        print("[Main] 错误: 没有生成任何网格")
        sys.exit(1)

    # 如果不用自定义法线，用简单面法线
    if not use_custom_normals and meshes:
        for mesh in meshes:
            mesh.normals = [(0.0, 1.0, 0.0)] * len(mesh.vertices)

    print()
    print("=" * 60)
    print("步骤 3/3: 合并导出 USD")
    print("=" * 60)

    merger = MeshMerger()
    merger.set_meshes(meshes)
    merger.set_materials(materials_list)
    merger.merge_and_export(output_file, mesh_prefix, use_custom_normals)

    # 统计
    total_vertices = sum(len(m.vertices) for m in meshes)
    total_faces = sum(len(m.faces) for m in meshes)

    print()
    print("=" * 60)
    print("转换完成!")
    print("=" * 60)
    print(f"输出文件: {output_file}")
    print(f"总顶点数: {total_vertices}")
    print(f"总面数: {total_faces}")
    print(f"材质数量: {len(materials_list)}")


if __name__ == "__main__":
    main()
