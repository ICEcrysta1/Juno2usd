# Changelog

本文件记录项目主要变更。

## [Unreleased]

### Added


### Breaking Changes


### Changed


### Fixed


## [0.2.0]

> 实现语言由 Python 整体切换为 Rust，USD 技术栈由 usd-core（pxr）切换为 openusd。
> 除语言迁移外，几何生成、法线语义与导出结构均有实质变化，产物与 0.1.2 不逐顶点一致。

### Added

- TUI（Ratatui）：无参数启动进入交互界面；Input 目录文件选择、右栏转换前统计（后台异步解析、旋转加载、按文件签名内存缓存、文件变更即失效）、转换进度与结果统计；中/英 i18n（依 `LC_ALL`/`LANG` 初始化）
- 连接感知接头法线混合：`flattenNormals`（Top/Bottom/Both）端点开关四态规则（双开互相平均、单开对齐关侧、双关不混合），共轴判据门控（仅端面对接混合，侧向连接跳过），只改对接环侧面法线，端盖硬边不变
- UsdSkel 完整导出：SkelRoot + Skeleton（joints/bindTransforms/restTransforms）+ SkelBindingAPI（jointIndices/weights）；关节层级沿零件树构建，零件刚性绑定
- 权威参数文件 `src/settings/defaults.yaml`（编译期内嵌）：截面周向分段、鼻锥轴向细分、默认半径、Strut 圆角、空心壁厚、锐边阈值
- 可选几何对照 JSON dump（CLI 第二参数），供跨版本数值核对
- `tests/src` 镜像测试树：43 个模块测试文件，`#[cfg(test)] #[path]` 挂载，覆盖算法/解析/法线/骨骼/导出/TUI
- 启动时自动创建 `Input/`、`Output/` 目录

### Breaking Changes

- 实现语言 Python → Rust（edition 2024，需 Rust ≥ 1.100，Windows x64 `x86_64-pc-windows-gnullvm` 工具链）；移除 Python 流水线与 `run_pipeline.bat`，产物为单一可执行文件（`cargo build --release --locked`）
- USD 读写改用 openusd / openusd-schemas（纯 Rust 实现，无 C++ 依赖），移除 usd-core（pxr）依赖；仅使用 UsdGeom / UsdShade / UsdSkel 三族 schema
- CLI 契约变更：`juno2usd.exe <input.xml> [对照.json] [output.usda]`；无参数进入 TUI 而非打印帮助；移除 `--radius-x/--radius-z/--segments/--no-normals/--keep-cache/--prefix/--skeleton` 等参数
- 算法参数不再经 CLI 传入，统一由内嵌 YAML（`defaults.yaml`）权威管理
- 移除阶段间 JSON 缓存（`cache/gen`、`cache/normals`、`materials.json`、`skeleton_data.json` 等），全流程内存执行
- 骨骼不再可选（原 `--skeleton` 开关）：始终构建关节并导出 UsdSkel；无关节时退化为按材质分组
- 导出结构变更：0.1.2 将全部零件合并为单一 Mesh；现按 (关节, 材质) 分组输出多个 Mesh，附 UsdPreviewSurface 与 st primvar，并做退化面过滤与顶点紧凑化
- 法线与三角化语义重写（见 Changed），空心件尖端几何与默认壁厚变化（见 Fixed）；着色硬边/平滑结果、顶点与面数均与 0.1.2 不同

### Changed

- 法线模块重写为零件无关的纯数学执行器，忠实 Blender "Smooth by Angle" 自动平滑：锐边阈值 40° → 60°（YAML 权威），smooth fan + 角点角度加权，非流形边自动锐边；替代 0.1.2 按零件类型的启发式法线（鼻锥径向法线、端盖 135°–170° 平滑阈值等）
- 端盖硬边独立 pass（平面法线），与侧面平滑分离，避免对接环非流形拆分
- 尖点法线区分两类：圆锥尖入射边强制锐边，保持尖锐棱锥；非圆锥缩 0 端仅顶点硬角取面法线，母线边不标锐，锥面光滑（消除整面放射状硬边）
- 方形截面（圆角全 0）所有侧面棱强制锐边，90° 硬边不被阈值误圆滑；接头混合整体跳过带硬角的环，圆角/圆环仍平滑
- 侧面三角化策略：先保留四边面，法线计算前统一优化三角化——塌缩四边形改单三角扇（消 sliver/阶梯），完整四边形选法线偏差最小的对角线（平局取较短）；鼻锥尖端直接发射三角扇
- 空心件（Inlet/Fairing）与空心鼻锥内尖点沿轴内缩至内壁半径归零处（fuselage 线性锥 recess = 壁厚 × 长 / 底半径；鼻锥按 t^2.1 剖面反解），内外尖点拓扑分离，保留尖端壁厚
- 骨骼绑定矩阵显式采用 USD/Gf 行向量约定（v·M：旋转块转置、平移在列），rest 逐级复合与 bind 自洽，Blender 中关节朝向正确、无需清空变换
- 导出 (关节, 材质) 分组保持首次出现顺序；材质名统一清洗
- 转换编排收口至 `core::convert`（parse → shapes → normal → blend → skel → usda），进度回调供 CLI/TUI 复用；重复几何数学 helper 合并至 `shapes/algo/transform`
- 版本号由 `env!(CARGO_PKG_VERSION)` 读取；版权持有者更新为 ICE_crystal，仓库统一至 GitHub `ICEcrysta1/Juno2usd`

### Fixed

- 空心鼻锥尖端内外层顶点合并导致尖端壁厚丢失（0.1.2 遗留）：内环点退化发射独立中心顶点，内尖点内缩（见 Changed）
- FairingBase1/Fairing1 默认壁厚 0 → 0.045，消除零壁厚空心整流罩
- 锥体空心底部端盖面误分类为侧面（补充 inner_ring 并集）
- 旧法线计算的同顶点多平滑组法线覆盖、端盖法线轴向回退遗漏（随法线模块重写消除）
- TUI 底部三组提示左/右对齐均分铺满整行
