//! 简体中文文案；键保持稳定，格式参数由编译器检查。

#[rustfmt::skip]
macro_rules! message {
    // 顶栏与右盒标签。
    (ui_title) => { "Juno2USD 版本: {}" };
    (ui_workspace) => { "工作目录" };
    (ui_choose) => { "选择" };
    (ui_info) => { "信息" };
    (ui_input) => { "输入" };
    (ui_output) => { "输出" };
    (ui_components) => { "面数" };
    (ui_skel) => { "骨骼" };
    (ui_size) => { "大小" };

    // 状态。
    (ui_idle) => { "空闲" };
    (ui_running) => { "转换中" };
    (ui_done) => { "完成" };
    (ui_failed) => { "失败" };

    // 提示。
    (ui_hint) => { "↑/↓ 选择   Enter 转换   q/Esc 退出" };
    (ui_hint_running) => { "↑/↓ 选择   Enter(忽略)   q/Esc 退出" };
    (ui_select_prompt) => { "选择文件后回车转换" };
    (ui_usage) => { "用法: juno2usd <input.xml> [dump.json] [output.usda]" };
}

pub(crate) use message;
