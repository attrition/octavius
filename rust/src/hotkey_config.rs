use std::ffi::{c_char, c_int, c_void};
use std::ptr::{self, addr_of, addr_of_mut};

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HotkeyAction {
    ArrowUp = 0,
    ArrowDown = 1,
    ArrowLeft = 2,
    ArrowRight = 3,
    TogglePause = 4,
    ToggleOverlay = 5,
    CycleLegion = 6,
    IncreaseGameSpeed = 7,
    DecreaseGameSpeed = 8,
    RotateMapLeft = 9,
    RotateMapRight = 10,
    BuildClearLand = 11,
    BuildVacantHouse = 12,
    BuildRoad = 13,
    BuildPlaza = 14,
    BuildGardens = 15,
    BuildPrefecture = 16,
    BuildEngineersPost = 17,
    BuildDoctor = 18,
    BuildGranary = 19,
    BuildWarehouse = 20,
    BuildMarket = 21,
    BuildWall = 22,
    BuildGatehouse = 23,
    BuildReservoir = 24,
    BuildAqueduct = 25,
    BuildFountain = 26,
    ShowAdvisorLabor = 27,
    ShowAdvisorMilitary = 28,
    ShowAdvisorImperial = 29,
    ShowAdvisorRatings = 30,
    ShowAdvisorTrade = 31,
    ShowAdvisorPopulation = 32,
    ShowAdvisorHealth = 33,
    ShowAdvisorEducation = 34,
    ShowAdvisorEntertainment = 35,
    ShowAdvisorReligion = 36,
    ShowAdvisorFinancial = 37,
    ShowAdvisorChief = 38,
    ShowOverlayWater = 39,
    ShowOverlayFire = 40,
    ShowOverlayDamage = 41,
    ShowOverlayCrime = 42,
    ShowOverlayProblems = 43,
    EditorToggleBattleInfo = 44,
    LoadFile = 45,
    SaveFile = 46,
    ToggleOctaviusUi = 47,
    GoToBookmark1 = 48,
    GoToBookmark2 = 49,
    GoToBookmark3 = 50,
    GoToBookmark4 = 51,
    SetBookmark1 = 52,
    SetBookmark2 = 53,
    SetBookmark3 = 54,
    SetBookmark4 = 55,
    CenterWindow = 56,
    ToggleFullscreen = 57,
    ResizeTo640 = 58,
    ResizeTo800 = 59,
    ResizeTo1024 = 60,
    ResizeTo720 = 61,
    ResizeTo1080 = 62,
    ResizeTo1440 = 63,
    SaveScreenshot = 64,
    SaveCityScreenshot = 65,
    BuildClone = 66,
    MaxItems = 67,
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KeyType {
    None = 0,
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9, Num0,
    Minus, Equals, Enter, Escape, Backspace, Tab, Space,
    LeftBracket, RightBracket, Backslash, Semicolon, Apostrophe, Grave, Comma, Period, Slash,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Insert, Delete, Home, End, PageUp, PageDown,
    Right, Left, Down, Up,
    Kp1, Kp2, Kp3, Kp4, Kp5, Kp6, Kp7, Kp8, Kp9, Kp0,
    KpPeriod, KpPlus, KpMinus, KpMultiply, KpDivide,
    NonUs,
    MaxItems,
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KeyModifierType {
    None = 0,
    Shift = 1,
    Ctrl = 2,
    Alt = 4,
    Gui = 8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct HotkeyMapping {
    pub key: KeyType,
    pub modifiers: KeyModifierType,
    pub action: HotkeyAction,
}

const MAX_MAPPINGS: usize = 67 * 2;
const MAX_LINE: usize = 100;

// Helper to make *const c_char Sync
#[repr(transparent)]
struct SyncPtr(*const c_char);
unsafe impl Sync for SyncPtr {}

static INI_FILENAME: SyncPtr = SyncPtr("octavius-hotkeys.ini\0".as_ptr() as *const c_char);

static INI_KEYS: [SyncPtr; 67] = [
    SyncPtr("arrow_up\0".as_ptr() as *const c_char),
    SyncPtr("arrow_down\0".as_ptr() as *const c_char),
    SyncPtr("arrow_left\0".as_ptr() as *const c_char),
    SyncPtr("arrow_right\0".as_ptr() as *const c_char),
    SyncPtr("toggle_pause\0".as_ptr() as *const c_char),
    SyncPtr("toggle_overlay\0".as_ptr() as *const c_char),
    SyncPtr("cycle_legion\0".as_ptr() as *const c_char),
    SyncPtr("increase_game_speed\0".as_ptr() as *const c_char),
    SyncPtr("decrease_game_speed\0".as_ptr() as *const c_char),
    SyncPtr("rotate_map_left\0".as_ptr() as *const c_char),
    SyncPtr("rotate_map_right\0".as_ptr() as *const c_char),
    SyncPtr("build_clear\0".as_ptr() as *const c_char),
    SyncPtr("build_house\0".as_ptr() as *const c_char),
    SyncPtr("build_road\0".as_ptr() as *const c_char),
    SyncPtr("build_plaza\0".as_ptr() as *const c_char),
    SyncPtr("build_gardens\0".as_ptr() as *const c_char),
    SyncPtr("build_prefecture\0".as_ptr() as *const c_char),
    SyncPtr("build_engineers_post\0".as_ptr() as *const c_char),
    SyncPtr("build_doctor\0".as_ptr() as *const c_char),
    SyncPtr("build_granary\0".as_ptr() as *const c_char),
    SyncPtr("build_warehouse\0".as_ptr() as *const c_char),
    SyncPtr("build_market\0".as_ptr() as *const c_char),
    SyncPtr("build_wall\0".as_ptr() as *const c_char),
    SyncPtr("build_gatehouse\0".as_ptr() as *const c_char),
    SyncPtr("build_reservoir\0".as_ptr() as *const c_char),
    SyncPtr("build_aqueduct\0".as_ptr() as *const c_char),
    SyncPtr("build_fountain\0".as_ptr() as *const c_char),
    SyncPtr("show_advisor_labor\0".as_ptr() as *const c_char),
    SyncPtr("show_advisor_military\0".as_ptr() as *const c_char),
    SyncPtr("show_advisor_imperial\0".as_ptr() as *const c_char),
    SyncPtr("show_advisor_ratings\0".as_ptr() as *const c_char),
    SyncPtr("show_advisor_trade\0".as_ptr() as *const c_char),
    SyncPtr("show_advisor_population\0".as_ptr() as *const c_char),
    SyncPtr("show_advisor_health\0".as_ptr() as *const c_char),
    SyncPtr("show_advisor_education\0".as_ptr() as *const c_char),
    SyncPtr("show_advisor_entertainment\0".as_ptr() as *const c_char),
    SyncPtr("show_advisor_religion\0".as_ptr() as *const c_char),
    SyncPtr("show_advisor_financial\0".as_ptr() as *const c_char),
    SyncPtr("show_advisor_chief\0".as_ptr() as *const c_char),
    SyncPtr("show_overlay_water\0".as_ptr() as *const c_char),
    SyncPtr("show_overlay_fire\0".as_ptr() as *const c_char),
    SyncPtr("show_overlay_damage\0".as_ptr() as *const c_char),
    SyncPtr("show_overlay_crime\0".as_ptr() as *const c_char),
    SyncPtr("show_overlay_problems\0".as_ptr() as *const c_char),
    SyncPtr("editor_toggle_battle_info\0".as_ptr() as *const c_char),
    SyncPtr("load_file\0".as_ptr() as *const c_char),
    SyncPtr("save_file\0".as_ptr() as *const c_char),
    SyncPtr("toggle_octavius_ui\0".as_ptr() as *const c_char),
    SyncPtr("go_to_bookmark_1\0".as_ptr() as *const c_char),
    SyncPtr("go_to_bookmark_2\0".as_ptr() as *const c_char),
    SyncPtr("go_to_bookmark_3\0".as_ptr() as *const c_char),
    SyncPtr("go_to_bookmark_4\0".as_ptr() as *const c_char),
    SyncPtr("set_bookmark_1\0".as_ptr() as *const c_char),
    SyncPtr("set_bookmark_2\0".as_ptr() as *const c_char),
    SyncPtr("set_bookmark_3\0".as_ptr() as *const c_char),
    SyncPtr("set_bookmark_4\0".as_ptr() as *const c_char),
    SyncPtr("center_screen\0".as_ptr() as *const c_char),
    SyncPtr("toggle_fullscreen\0".as_ptr() as *const c_char),
    SyncPtr("resize_to_640\0".as_ptr() as *const c_char),
    SyncPtr("resize_to_800\0".as_ptr() as *const c_char),
    SyncPtr("resize_to_1024\0".as_ptr() as *const c_char),
    SyncPtr("resize_to_720\0".as_ptr() as *const c_char),
    SyncPtr("resize_to_1080\0".as_ptr() as *const c_char),
    SyncPtr("resize_to_1440\0".as_ptr() as *const c_char),
    SyncPtr("save_screenshot\0".as_ptr() as *const c_char),
    SyncPtr("save_city_screenshot\0".as_ptr() as *const c_char),
    SyncPtr("clone_building\0".as_ptr() as *const c_char),
];

struct HotkeyData {
    default_mappings: [[HotkeyMapping; 2]; 67],
    mappings: [HotkeyMapping; MAX_MAPPINGS],
    num_mappings: c_int,
}

static mut DATA: HotkeyData = unsafe { std::mem::zeroed() };

unsafe extern "C" {
    fn file_open(filename: *const c_char, mode: *const c_char) -> *mut c_void;
    fn file_close(stream: *mut c_void) -> c_int;
    fn log_info(msg: *const c_char, param_str: *const c_char, param_int: c_int);
    fn log_error(msg: *const c_char, param_str: *const c_char, param_int: c_int);
    
    fn system_keyboard_key_for_symbol(name: *const c_char) -> KeyType;
    fn key_combination_from_name(name: *const c_char, key: *mut KeyType, modifiers: *mut KeyModifierType) -> c_int;
    fn key_combination_name(key: KeyType, modifiers: KeyModifierType) -> *const c_char;
    fn hotkey_install_mapping(mappings: *const HotkeyMapping, num_mappings: c_int);

    fn fgets(s: *mut c_char, n: c_int, stream: *mut c_void) -> *mut c_char;
    fn io_fprintf(stream: *mut c_void, format: *const c_char, ...) -> c_int;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
}

unsafe fn set_mapping(key: KeyType, modifiers: KeyModifierType, action: HotkeyAction) {
    unsafe {
        let mappings = addr_of_mut!(DATA.default_mappings[action as usize]);
        let mapping0 = &mut (*mappings)[0];
        let mut mapping = mapping0;
        if mapping.key != KeyType::None {
            mapping = &mut (*mappings)[1];
        }
        if mapping.key != KeyType::None {
            return;
        }
        mapping.key = key;
        mapping.modifiers = modifiers;
        mapping.action = action;
    }
}

unsafe fn set_layout_mapping(name: *const c_char, default_key: KeyType, modifiers: KeyModifierType, action: HotkeyAction) {
    unsafe {
        let mut key = system_keyboard_key_for_symbol(name);
        if key == KeyType::None {
            log_info("No key found on layout for\0".as_ptr() as *const c_char, name, 0);
            key = default_key;
        }
        set_mapping(key, modifiers, action);
    }
}

unsafe fn init_defaults() {
    unsafe {
        ptr::write_bytes(addr_of_mut!(DATA.default_mappings), 0, 1);
        
        set_mapping(KeyType::Up, KeyModifierType::None, HotkeyAction::ArrowUp);
        set_mapping(KeyType::Down, KeyModifierType::None, HotkeyAction::ArrowDown);
        set_mapping(KeyType::Left, KeyModifierType::None, HotkeyAction::ArrowLeft);
        set_mapping(KeyType::Right, KeyModifierType::None, HotkeyAction::ArrowRight);
        
        set_layout_mapping("P\0".as_ptr() as *const c_char, KeyType::P, KeyModifierType::None, HotkeyAction::TogglePause);
        set_mapping(KeyType::Space, KeyModifierType::None, HotkeyAction::ToggleOverlay);
        set_layout_mapping("L\0".as_ptr() as *const c_char, KeyType::L, KeyModifierType::None, HotkeyAction::CycleLegion);
        set_layout_mapping("[\0".as_ptr() as *const c_char, KeyType::LeftBracket, KeyModifierType::None, HotkeyAction::DecreaseGameSpeed);
        set_layout_mapping("]\0".as_ptr() as *const c_char, KeyType::RightBracket, KeyModifierType::None, HotkeyAction::IncreaseGameSpeed);
        
        set_mapping(KeyType::PageDown, KeyModifierType::None, HotkeyAction::DecreaseGameSpeed);
        set_mapping(KeyType::PageUp, KeyModifierType::None, HotkeyAction::IncreaseGameSpeed);
        set_mapping(KeyType::Home, KeyModifierType::None, HotkeyAction::RotateMapLeft);
        set_mapping(KeyType::End, KeyModifierType::None, HotkeyAction::RotateMapRight);
        
        set_mapping(KeyType::Num1, KeyModifierType::None, HotkeyAction::ShowAdvisorLabor);
        set_mapping(KeyType::Num2, KeyModifierType::None, HotkeyAction::ShowAdvisorMilitary);
        set_mapping(KeyType::Num3, KeyModifierType::None, HotkeyAction::ShowAdvisorImperial);
        set_mapping(KeyType::Num4, KeyModifierType::None, HotkeyAction::ShowAdvisorRatings);
        set_mapping(KeyType::Num5, KeyModifierType::None, HotkeyAction::ShowAdvisorTrade);
        set_mapping(KeyType::Num6, KeyModifierType::None, HotkeyAction::ShowAdvisorPopulation);
        set_mapping(KeyType::Num7, KeyModifierType::None, HotkeyAction::ShowAdvisorHealth);
        set_mapping(KeyType::Num8, KeyModifierType::None, HotkeyAction::ShowAdvisorEducation);
        set_mapping(KeyType::Num9, KeyModifierType::None, HotkeyAction::ShowAdvisorEntertainment);
        set_mapping(KeyType::Num0, KeyModifierType::None, HotkeyAction::ShowAdvisorReligion);
        
        set_mapping(KeyType::Kp1, KeyModifierType::None, HotkeyAction::ShowAdvisorLabor);
        set_mapping(KeyType::Kp2, KeyModifierType::None, HotkeyAction::ShowAdvisorMilitary);
        set_mapping(KeyType::Kp3, KeyModifierType::None, HotkeyAction::ShowAdvisorImperial);
        set_mapping(KeyType::Kp4, KeyModifierType::None, HotkeyAction::ShowAdvisorRatings);
        set_mapping(KeyType::Kp5, KeyModifierType::None, HotkeyAction::ShowAdvisorTrade);
        set_mapping(KeyType::Kp6, KeyModifierType::None, HotkeyAction::ShowAdvisorPopulation);
        set_mapping(KeyType::Kp7, KeyModifierType::None, HotkeyAction::ShowAdvisorHealth);
        set_mapping(KeyType::Kp8, KeyModifierType::None, HotkeyAction::ShowAdvisorEducation);
        set_mapping(KeyType::Kp9, KeyModifierType::None, HotkeyAction::ShowAdvisorEntertainment);
        set_mapping(KeyType::Kp0, KeyModifierType::None, HotkeyAction::ShowAdvisorReligion);
        
        set_layout_mapping("-\0".as_ptr() as *const c_char, KeyType::Minus, KeyModifierType::None, HotkeyAction::ShowAdvisorFinancial);
        set_layout_mapping("=\0".as_ptr() as *const c_char, KeyType::Equals, KeyModifierType::None, HotkeyAction::ShowAdvisorChief);
        set_layout_mapping("W\0".as_ptr() as *const c_char, KeyType::W, KeyModifierType::None, HotkeyAction::ShowOverlayWater);
        set_layout_mapping("F\0".as_ptr() as *const c_char, KeyType::F, KeyModifierType::None, HotkeyAction::ShowOverlayFire);
        set_layout_mapping("D\0".as_ptr() as *const c_char, KeyType::D, KeyModifierType::None, HotkeyAction::ShowOverlayDamage);
        set_layout_mapping("C\0".as_ptr() as *const c_char, KeyType::C, KeyModifierType::None, HotkeyAction::ShowOverlayCrime);
        set_layout_mapping("T\0".as_ptr() as *const c_char, KeyType::T, KeyModifierType::None, HotkeyAction::ShowOverlayProblems);
        
        set_layout_mapping("A\0".as_ptr() as *const c_char, KeyType::A, KeyModifierType::Ctrl, HotkeyAction::EditorToggleBattleInfo);
        set_layout_mapping("O\0".as_ptr() as *const c_char, KeyType::O, KeyModifierType::Ctrl, HotkeyAction::LoadFile);
        set_layout_mapping("S\0".as_ptr() as *const c_char, KeyType::S, KeyModifierType::Ctrl, HotkeyAction::SaveFile);
        
        set_mapping(KeyType::F1, KeyModifierType::None, HotkeyAction::GoToBookmark1);
        set_mapping(KeyType::F2, KeyModifierType::None, HotkeyAction::GoToBookmark2);
        set_mapping(KeyType::F3, KeyModifierType::None, HotkeyAction::GoToBookmark3);
        set_mapping(KeyType::F4, KeyModifierType::None, HotkeyAction::GoToBookmark4);
        
        set_mapping(KeyType::F1, KeyModifierType::Ctrl, HotkeyAction::SetBookmark1);
        set_mapping(KeyType::F2, KeyModifierType::Ctrl, HotkeyAction::SetBookmark2);
        set_mapping(KeyType::F3, KeyModifierType::Ctrl, HotkeyAction::SetBookmark3);
        set_mapping(KeyType::F4, KeyModifierType::Ctrl, HotkeyAction::SetBookmark4);
        
        set_mapping(KeyType::F1, KeyModifierType::Alt, HotkeyAction::SetBookmark1);
        set_mapping(KeyType::F2, KeyModifierType::Alt, HotkeyAction::SetBookmark2);
        set_mapping(KeyType::F3, KeyModifierType::Alt, HotkeyAction::SetBookmark3);
        set_mapping(KeyType::F4, KeyModifierType::Alt, HotkeyAction::SetBookmark4);
        
        set_mapping(KeyType::F5, KeyModifierType::None, HotkeyAction::CenterWindow);
        set_mapping(KeyType::F6, KeyModifierType::None, HotkeyAction::ToggleFullscreen);
        set_mapping(KeyType::Enter, KeyModifierType::Alt, HotkeyAction::ToggleFullscreen);
        
        set_mapping(KeyType::F7, KeyModifierType::None, HotkeyAction::ResizeTo640);
        set_mapping(KeyType::F8, KeyModifierType::None, HotkeyAction::ResizeTo800);
        set_mapping(KeyType::F9, KeyModifierType::None, HotkeyAction::ResizeTo1024);
        set_mapping(KeyType::F10, KeyModifierType::None, HotkeyAction::ResizeTo720);
        set_mapping(KeyType::F11, KeyModifierType::None, HotkeyAction::ResizeTo1080);
        set_mapping(KeyType::F12, KeyModifierType::None, HotkeyAction::ResizeTo1440);
        
        set_mapping(KeyType::F12, KeyModifierType::Shift, HotkeyAction::SaveScreenshot);
        set_mapping(KeyType::F12, KeyModifierType::Alt, HotkeyAction::SaveScreenshot);
        set_mapping(KeyType::F12, KeyModifierType::Ctrl, HotkeyAction::SaveCityScreenshot);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hotkey_for_action(action: HotkeyAction, index: c_int) -> *const HotkeyMapping {
    unsafe {
        let mut num = 0;
        for i in 0..(*addr_of!(DATA.num_mappings)) {
            let m = (addr_of!(DATA.mappings) as *const HotkeyMapping).add(i as usize);
            if (*m).action == action {
                if num == index {
                    return m;
                }
                num += 1;
            }
        }
        ptr::null()
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hotkey_default_for_action(action: HotkeyAction, index: c_int) -> *const HotkeyMapping {
    unsafe {
        if index < 0 || index >= 2 || (action as i32) < 0 || action as i32 >= HotkeyAction::MaxItems as i32 {
            return ptr::null();
        }
        addr_of!(DATA.default_mappings[action as usize][index as usize])
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hotkey_config_clear() {
    unsafe {
        (*addr_of_mut!(DATA.num_mappings)) = 0;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hotkey_config_add_mapping(mapping: *const HotkeyMapping) {
    unsafe {
        if *addr_of!(DATA.num_mappings) < MAX_MAPPINGS as c_int {
            let m_ptr = (addr_of_mut!(DATA.mappings) as *mut HotkeyMapping).add(*addr_of!(DATA.num_mappings) as usize);
            *m_ptr = *mapping;
            (*addr_of_mut!(DATA.num_mappings)) += 1;
        }
    }
}

unsafe fn load_defaults() {
    unsafe {
        hotkey_config_clear();
        for action in 0..HotkeyAction::MaxItems as usize {
            for index in 0..2 {
                let m = addr_of!(DATA.default_mappings[action][index]);
                if (*m).key != KeyType::None {
                    hotkey_config_add_mapping(m);
                }
            }
        }
    }
}

unsafe fn add_mapping(hotkey_id: c_int, value: *const c_char) {
    unsafe {
        let mut mapping: HotkeyMapping = std::mem::zeroed();
        if key_combination_from_name(value, &mut mapping.key, &mut mapping.modifiers) != 0 {
            mapping.action = std::mem::transmute(hotkey_id);
            hotkey_config_add_mapping(&mapping);
        }
    }
}

unsafe fn load_file() {
    unsafe {
        hotkey_config_clear();
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
            if equals.is_null() {
                continue;
            }
            *equals = 0;
            let key_ptr = line_buffer.as_ptr();
            let value_ptr = equals.add(1);
            
            for i in 0..HotkeyAction::MaxItems as usize {
                if !INI_KEYS[i].0.is_null() && strcmp(INI_KEYS[i].0, key_ptr) == 0 {
                    add_mapping(i as c_int, value_ptr);
                    break;
                }
            }
            // Migrate changed keys
            if strcmp("build_clear_land\0".as_ptr() as *const c_char, key_ptr) == 0 {
                add_mapping(HotkeyAction::BuildVacantHouse as c_int, value_ptr);
            } else if strcmp("build_vacant_house\0".as_ptr() as *const c_char, key_ptr) == 0 {
                add_mapping(HotkeyAction::BuildClearLand as c_int, value_ptr);
            }
        }
        file_close(fp);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hotkey_config_load() {
    unsafe {
        init_defaults();
        load_file();
        if *addr_of!(DATA.num_mappings) == 0 {
            load_defaults();
        }
        hotkey_install_mapping(addr_of!(DATA.mappings) as *const HotkeyMapping, *addr_of!(DATA.num_mappings));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hotkey_config_save() {
    unsafe {
        hotkey_install_mapping(addr_of!(DATA.mappings) as *const HotkeyMapping, *addr_of!(DATA.num_mappings));
        let fp = file_open(INI_FILENAME.0, "wt\0".as_ptr() as *const c_char);
        if fp.is_null() {
            log_error("Unable to write hotkey configuration file\0".as_ptr() as *const c_char, INI_FILENAME.0, 0);
            return;
        }
        for i in 0..(*addr_of!(DATA.num_mappings)) {
            let m = (addr_of!(DATA.mappings) as *const HotkeyMapping).add(i as usize);
            let key_name = key_combination_name((*m).key, (*m).modifiers);
            io_fprintf(fp, "%s=%s\n\0".as_ptr() as *const c_char, INI_KEYS[(*m).action as usize].0, key_name);
        }
        file_close(fp);
    }
}
