//! English strings; keys stay stable, format args checked by the compiler.

#[rustfmt::skip]
macro_rules! message {
    // Top bar and right-panel labels.
    (ui_title) => { "Juno2USD Version: {}" };
    (ui_workspace) => { "WorkSpace" };
    (ui_choose) => { "Choose" };
    (ui_info) => { "info" };
    (ui_input) => { "Input" };
    (ui_output) => { "Output" };
    (ui_components) => { "Components" };
    (ui_skel) => { "Skel" };
    (ui_size) => { "Size" };

    // Status.
    (ui_idle) => { "Idle" };
    (ui_running) => { "Converting" };
    (ui_done) => { "Done" };
    (ui_failed) => { "Failed" };

    // Hints.
    (ui_hint) => { "↑/↓ select   Enter convert   q/Esc quit" };
    (ui_hint_running) => { "↑/↓ select   Enter(ignored)   q/Esc quit" };
    (ui_select_prompt) => { "Select a file and press Enter to convert" };
    (ui_usage) => { "usage: juno2usd <input.xml> [dump.json] [output.usda]" };
}

pub(crate) use message;
