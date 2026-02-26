use std::ffi::{c_char, c_int, c_void};

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ConfigKey {
    GameplayFixImmigrationBug = 0,
    GameplayFix100YearGhosts = 1,
    ScreenDisplayScale = 2,
    ScreenCursorScale = 3,
    UiOctaviusUi = 4,
    UiSidebarInfo = 5,
    UiShowIntroVideo = 6,
    UiSmoothScrolling = 7,
    UiDisableMouseEdgeScrolling = 8,
    UiDisableRightClickMapDrag = 9,
    UiInverseRightClickMapDrag = 10,
    UiVisualFeedbackOnDelete = 11,
    UiAllowCyclingTemples = 12,
    UiShowWaterStructureRange = 13,
    UiShowConstructionSize = 14,
    UiHighlightLegions = 15,
    UiShowMilitarySidebar = 16,
    UiShowSpeedrunInfo = 17,
    MaxEntries = 18,
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ConfigStringKey {
    UiLanguageDir = 0,
    MaxEntries = 1,
}

const CONFIG_STRING_VALUE_MAX: usize = 64;
const MAX_LINE: usize = 100;

// Helper to make *const c_char Sync
#[repr(transparent)]
struct SyncPtr(*const c_char);
unsafe impl Sync for SyncPtr {}

static INI_FILENAME: SyncPtr = SyncPtr("octavius.ini\0".as_ptr() as *const c_char);

static INI_KEYS: [SyncPtr; 18] = [
    SyncPtr("gameplay_fix_immigration\0".as_ptr() as *const c_char),
    SyncPtr("gameplay_fix_100y_ghosts\0".as_ptr() as *const c_char),
    SyncPtr("screen_display_scale\0".as_ptr() as *const c_char),
    SyncPtr("screen_cursor_scale\0".as_ptr() as *const c_char),
    SyncPtr("ui_octavius_ui\0".as_ptr() as *const c_char),
    SyncPtr("ui_sidebar_info\0".as_ptr() as *const c_char),
    SyncPtr("ui_show_intro_video\0".as_ptr() as *const c_char),
    SyncPtr("ui_smooth_scrolling\0".as_ptr() as *const c_char),
    SyncPtr("ui_disable_mouse_edge_scrolling\0".as_ptr() as *const c_char),
    SyncPtr("ui_disable_map_drag\0".as_ptr() as *const c_char),
    SyncPtr("ui_inverse_map_drag\0".as_ptr() as *const c_char),
    SyncPtr("ui_visual_feedback_on_delete\0".as_ptr() as *const c_char),
    SyncPtr("ui_allow_cycling_temples\0".as_ptr() as *const c_char),
    SyncPtr("ui_show_water_structure_range\0".as_ptr() as *const c_char),
    SyncPtr("ui_show_construction_size\0".as_ptr() as *const c_char),
    SyncPtr("ui_highlight_legions\0".as_ptr() as *const c_char),
    SyncPtr("ui_show_military_sidebar\0".as_ptr() as *const c_char),
    SyncPtr("ui_show_speedrun_info\0".as_ptr() as *const c_char),
];

static INI_STRING_KEYS: [SyncPtr; 1] = [
    SyncPtr("ui_language_dir\0".as_ptr() as *const c_char),
];

static mut VALUES: [c_int; 18] = [0; 18];
static mut STRING_VALUES: [[c_char; CONFIG_STRING_VALUE_MAX]; 1] = [[0; CONFIG_STRING_VALUE_MAX]; 1];

static DEFAULT_VALUES: [c_int; 18] = {
    let mut v = [0; 18];
    v[2] = 100; // ScreenDisplayScale
    v[3] = 100; // ScreenCursorScale
    v
};

unsafe extern "C" {
    fn file_open(filename: *const c_char, mode: *const c_char) -> *mut c_void;
    fn file_close(stream: *mut c_void) -> c_int;
    fn log_info(msg: *const c_char, param_str: *const c_char, param_int: c_int);
    fn log_error(msg: *const c_char, param_str: *const c_char, param_int: c_int);
    
    // Standard C library functions
    fn fgets(s: *mut c_char, n: c_int, stream: *mut c_void) -> *mut c_char;
    fn io_fprintf(stream: *mut c_void, format: *const c_char, ...) -> c_int;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strncpy(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
    fn atoi(s: *const c_char) -> c_int;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn config_get(key: ConfigKey) -> c_int {
    unsafe { VALUES[key as usize] }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn config_set(key: ConfigKey, value: c_int) {
    unsafe { VALUES[key as usize] = value; }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn config_get_string(key: ConfigStringKey) -> *const c_char {
    unsafe { STRING_VALUES[key as usize].as_ptr() }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn config_set_string(key: ConfigStringKey, value: *const c_char) {
    unsafe {
        if value.is_null() {
            STRING_VALUES[key as usize][0] = 0;
        } else {
            strncpy(STRING_VALUES[key as usize].as_mut_ptr(), value, CONFIG_STRING_VALUE_MAX - 1);
            STRING_VALUES[key as usize][CONFIG_STRING_VALUE_MAX - 1] = 0;
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn config_get_default_value(key: ConfigKey) -> c_int {
    DEFAULT_VALUES[key as usize]
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn config_get_default_string_value(_key: ConfigStringKey) -> *const c_char {
    "\0".as_ptr() as *const c_char
}

unsafe fn set_defaults() {
    unsafe {
        for i in 0..18 {
            VALUES[i] = DEFAULT_VALUES[i];
        }
        STRING_VALUES[0][0] = 0;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn config_load() {
    unsafe {
        set_defaults();
        let fp = file_open(INI_FILENAME.0, "rt\0".as_ptr() as *const c_char);
        if fp.is_null() {
            return;
        }
        let mut line_buffer = [0 as c_char; MAX_LINE];
        while !fgets(line_buffer.as_mut_ptr(), MAX_LINE as c_int, fp).is_null() {
            let mut size = strlen(line_buffer.as_ptr());
            while size > 0 && (line_buffer[size - 1] == b'\n' as c_char || line_buffer[size - 1] == b'\r' as c_char) {
                size -= 1;
                line_buffer[size] = 0;
            }
            let equals = strchr(line_buffer.as_ptr(), b'=' as c_int);
            if !equals.is_null() {
                *equals = 0;
                let key_ptr = line_buffer.as_ptr();
                let value_ptr = equals.add(1);
                
                let mut found = false;
                for i in 0..18 {
                    if strcmp(INI_KEYS[i].0, key_ptr) == 0 {
                        let val = atoi(value_ptr);
                        log_info("Config key\0".as_ptr() as *const c_char, INI_KEYS[i].0, val);
                        VALUES[i] = val;
                        found = true;
                        break;
                    }
                }
                if !found {
                    for i in 0..1 {
                        if strcmp(INI_STRING_KEYS[i].0, key_ptr) == 0 {
                            log_info("Config key\0".as_ptr() as *const c_char, INI_STRING_KEYS[i].0, 0);
                            log_info("Config value\0".as_ptr() as *const c_char, value_ptr, 0);
                            strncpy(STRING_VALUES[i].as_mut_ptr(), value_ptr, CONFIG_STRING_VALUE_MAX - 1);
                            STRING_VALUES[i][CONFIG_STRING_VALUE_MAX - 1] = 0;
                            break;
                        }
                    }
                }
            }
        }
        file_close(fp);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn config_save() {
    unsafe {
        let fp = file_open(INI_FILENAME.0, "wt\0".as_ptr() as *const c_char);
        if fp.is_null() {
            log_error("Unable to write configuration file\0".as_ptr() as *const c_char, INI_FILENAME.0, 0);
            return;
        }
        for i in 0..18 {
            io_fprintf(fp, "%s=%d\n\0".as_ptr() as *const c_char, INI_KEYS[i].0, VALUES[i]);
        }
        for i in 0..1 {
            io_fprintf(fp, "%s=%s\n\0".as_ptr() as *const c_char, INI_STRING_KEYS[i].0, STRING_VALUES[i].as_ptr());
        }
        file_close(fp);
    }
}
