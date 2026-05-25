# 更新说明

## [未发布] — 法线计算架构重构 (feature/normal-refactor)

### 重构
- **normal.py**: 713行 → 140行，删除 `_compute_cylinder/_compute_cone/_angle_split_normals/_smooth_group_normals/_compute_cap_radial/_compute_simple`，`compute()` 退化为纯数学执行器，只根据 `ring_info.smooth_groups` 执行法线计算
- **_geometry.py**: 删除重复的 `_calc_face_normal`，统一使用 `utils/math3d.calc_face_normal`
- **math3d.py**: 新增 `calc_face_normal` 纯数学工具函数
- **models.py**: `FuselageParams` 新增 `flatten_normals` 字段
- **parser.py**: 解析 XML `flattenNormals` 属性

### 新增
- `flatten_normals` 支持：零件根据 XML 属性决定端盖法线是否扁平
  - 扁平端盖 → 独立平滑组（硬边）
  - 非扁平端盖 → 融入侧面平滑组（为零件间连接预留光滑过渡）
- 圆柱/锥体端盖面和内侧面正确分类到独立平滑组

### 修复
- `_calc_face_normal` 去重（之前在 normal.py 和 _geometry.py 各有一份）
- `classify_smooth_groups` 和 `_angle_split_normals` 的锐边检测逻辑去重

## [已发布] — 平滑组按零件分类 (feature/smooth-groups-by-part)

### 重构
- 法线硬边改为零件决定：`smooth_groups` 由几何代码预计算，NormalService 直接使用
- 新增 `classify_smooth_groups` 通用函数（角度检测 + 洪水填充）
- `_normals_by_groups` 使用 (vertex, group_id) 双键支持硬边多法线

### 修复
- 同顶点多平滑组法线覆盖问题
- 锥体 hollow 底部端盖面误分类为侧面
- `_normals_by_groups` 返回类型注解修正
