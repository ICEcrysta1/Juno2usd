//! 简体中文文案；键保持稳定，格式参数由编译器检查。

#[rustfmt::skip]
macro_rules! message {
    // 顶栏与盒标题。
    (ui_title) => { "Juno2USD 版本: {}" };
    (ui_workspace) => { "工作目录" };
    (ui_choose) => { "选择" };
    (ui_info) => { "信息" };

    // 右盒标签。
    (ui_input) => { "输入" };
    (ui_output) => { "输出" };
    (ui_components) => { "面数" };
    (ui_skel) => { "骨骼" };
    (ui_size) => { "大小" };

    // 状态与底部按键。
    (ui_running) => { "转换中" };
    (ui_key_select) => { "↑/↓ 选择" };
    (ui_key_convert) => { "Enter 转换" };
    (ui_key_quit) => { "q/Esc 退出" };
}

pub(crate) use message;
