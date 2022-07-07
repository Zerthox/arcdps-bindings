use crate::{api::CombatEvent, imgui::sys::ImVec4, instance::ARC_INSTANCE};

/// Retrieve wide char path to Arc ini config file.
pub unsafe fn e0_config_path() -> *mut u16 {
    (ARC_INSTANCE.as_ref().unwrap().e0)()
}

/// Log a string to `arcdps.log`.
pub unsafe fn e3_log(string: *mut u8) {
    (ARC_INSTANCE.as_ref().unwrap().e3)(string)
}

/// Retrieve color pointers as array. Writes to buffer.
pub unsafe fn e5_colors(buffer: *mut [*mut ImVec4; 5]) {
    (ARC_INSTANCE.as_ref().unwrap().e5)(buffer)
}

/// Retrieve bit mask of current Arc UI settings.
pub unsafe fn e6_ui_settings() -> u64 {
    (ARC_INSTANCE.as_ref().unwrap().e6)()
}

/// Retrieve modifier keys as virtual key codes.
pub unsafe fn e7_modifiers() -> u64 {
    (ARC_INSTANCE.as_ref().unwrap().e7)()
}

/// Log a string to the logger window.
///
/// Colors are HTML-like: `<c=#aaaaaa>colored text</c>`.
pub unsafe fn e8_log_window(string: *mut u8) {
    (ARC_INSTANCE.as_ref().unwrap().e8)(string)
}

/// Add a [`CombatEvent`] to Arc's event processing.
///
/// `is_statechange` will be set to extension, pad61-64 will be set to sig.
/// Event will end up processed like ArcDPS events and logged to EVTC.
pub unsafe fn e9_add_event(event: *mut CombatEvent, sig: u32) {
    (ARC_INSTANCE.as_ref().unwrap().e9)(event, sig)
}