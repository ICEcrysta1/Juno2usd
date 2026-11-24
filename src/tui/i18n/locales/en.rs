//! English strings; keys stay stable, format args checked by the compiler.

#[rustfmt::skip]
macro_rules! message {
    // Top bar and box titles.
    (ui_title) => { "Juno2USD Version: {}" };
    (ui_workspace) => { "WorkSpace" };
    (ui_choose) => { "Choose" };
    (ui_info) => { "Info" };

    // Right-panel labels.
    (ui_input) => { "Input" };
    (ui_output) => { "Output" };
    (ui_parts) => { "Parts" };
    (ui_connections) => { "Connections" };
    (ui_materials) => { "Materials" };
    (ui_size) => { "Size" };
    (ui_faces) => { "Faces" };
    (ui_skel) => { "Skel" };

    // Status and bottom keys.
    (ui_running) => { "Converting" };
    (ui_key_select) => { "↑/↓ select" };
    (ui_key_convert) => { "Enter convert" };
    (ui_key_quit) => { "q/Esc quit" };
}

pub(crate) use message;

#[cfg(test)]
#[path = "../../../../tests/src/tui/i18n/locales/en_test.rs"]
mod tests;
