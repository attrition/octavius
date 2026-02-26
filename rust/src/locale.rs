use std::ffi::{c_char, c_int};
use crate::string::string_equals;

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LanguageType {
    Unknown = 0,
    English = 1,
    French = 2,
    German = 3,
    Italian = 4,
    Spanish = 5,
    Japanese = 6,
    Korean = 7,
    Polish = 8,
    Portuguese = 9,
    Russian = 10,
    Swedish = 11,
    SimplifiedChinese = 12,
    TraditionalChinese = 13,
    Czech = 14,
    Greek = 15,
    MaxItems = 16,
}

unsafe extern "C" {
    fn lang_get_string(group: c_int, index: c_int) -> *const u8;
    fn log_info(msg: *const c_char, param_str: *const c_char, param_int: c_int);
}

static NEW_GAME_ENGLISH: &[u8] = &[0x4e, 0x65, 0x77, 0x20, 0x47, 0x61, 0x6d, 0x65, 0];
static NEW_GAME_FRENCH: &[u8] = &[0x4e, 0x6f, 0x75, 0x76, 0x65, 0x6c, 0x6c, 0x65, 0x20, 0x70, 0x61, 0x72, 0x74, 0x69, 0x65, 0];
static NEW_GAME_GERMAN: &[u8] = &[0x4e, 0x65, 0x75, 0x65, 0x73, 0x20, 0x53, 0x70, 0x69, 0x65, 0x6c, 0];
static NEW_GAME_GREEK: &[u8] = &[0xcd, 0xdd, 0xef, 0x20, 0xd0, 0xe1, 0xe9, 0xf7, 0xed, 0xdf, 0xe4, 0xe9, 0];
static NEW_GAME_ITALIAN: &[u8] = &[0x4e, 0x75, 0x6f, 0x76, 0x61, 0x20, 0x70, 0x61, 0x72, 0x74, 0x69, 0x74, 0x61, 0];
static NEW_GAME_SPANISH: &[u8] = &[0x4e, 0x75, 0x65, 0x76, 0x61, 0x20, 0x70, 0x61, 0x72, 0x74, 0x69, 0x64, 0x61, 0];
static NEW_GAME_PORTUGUESE: &[u8] = &[0x4e, 0x6f, 0x76, 0x6f, 0x20, 0x6a, 0x6f, 0x67, 0x6f, 0];
static NEW_GAME_POLISH: &[u8] = &[0x4e, 0x6f, 0x77, 0x61, 0x20, 0x67, 0x72, 0x61, 0];
static NEW_GAME_RUSSIAN: &[u8] = &[0xcd, 0xee, 0xe2, 0xe0, 0xff, 0x20, 0xe8, 0xe3, 0xf0, 0xe0, 0];
static NEW_GAME_SWEDISH: &[u8] = &[0x4e, 0x79, 0x74, 0x74, 0x20, 0x73, 0x70, 0x65, 0x6c, 0];
static NEW_GAME_TRADITIONAL_CHINESE: &[u8] = &[0x83, 0x80, 0x20, 0x84, 0x80, 0x20, 0x85, 0x80, 0];
static NEW_GAME_SIMPLIFIED_CHINESE: &[u8] = &[0x82, 0x80, 0x20, 0x83, 0x80, 0x20, 0x84, 0x80, 0];
static NEW_GAME_KOREAN: &[u8] = &[0xbb, 0xf5, 0x20, 0xb0, 0xd4, 0xc0, 0xd3, 0];
static NEW_GAME_JAPANESE: &[u8] = &[0x83, 0x6a, 0x83, 0x85, 0x81, 0x5b, 0x83, 0x51, 0x81, 0x5b, 0x83, 0x80, 0];
static NEW_GAME_CZECH: &[u8] = &[0x4e, 0x6f, 0x76, 0xe1, 0x20, 0x68, 0x72, 0x61, 0];

struct LocaleData {
    last_determined_language: LanguageType,
}

static mut DATA: LocaleData = LocaleData {
    last_determined_language: LanguageType::Unknown,
};

unsafe fn determine_language() -> LanguageType {
    let new_game_string = unsafe { lang_get_string(1, 1) };
    if new_game_string.is_null() {
        return LanguageType::Unknown;
    }

    if unsafe { string_equals(NEW_GAME_ENGLISH.as_ptr(), new_game_string) } != 0 {
        LanguageType::English
    } else if unsafe { string_equals(NEW_GAME_FRENCH.as_ptr(), new_game_string) } != 0 {
        LanguageType::French
    } else if unsafe { string_equals(NEW_GAME_GERMAN.as_ptr(), new_game_string) } != 0 {
        LanguageType::German
    } else if unsafe { string_equals(NEW_GAME_GREEK.as_ptr(), new_game_string) } != 0 {
        LanguageType::Greek
    } else if unsafe { string_equals(NEW_GAME_ITALIAN.as_ptr(), new_game_string) } != 0 {
        LanguageType::Italian
    } else if unsafe { string_equals(NEW_GAME_SPANISH.as_ptr(), new_game_string) } != 0 {
        LanguageType::Spanish
    } else if unsafe { string_equals(NEW_GAME_PORTUGUESE.as_ptr(), new_game_string) } != 0 {
        LanguageType::Portuguese
    } else if unsafe { string_equals(NEW_GAME_POLISH.as_ptr(), new_game_string) } != 0 {
        LanguageType::Polish
    } else if unsafe { string_equals(NEW_GAME_RUSSIAN.as_ptr(), new_game_string) } != 0 {
        LanguageType::Russian
    } else if unsafe { string_equals(NEW_GAME_SWEDISH.as_ptr(), new_game_string) } != 0 {
        LanguageType::Swedish
    } else if unsafe { string_equals(NEW_GAME_CZECH.as_ptr(), new_game_string) } != 0 {
        LanguageType::Czech
    } else if unsafe { string_equals(NEW_GAME_TRADITIONAL_CHINESE.as_ptr(), new_game_string) } != 0 {
        LanguageType::TraditionalChinese
    } else if unsafe { string_equals(NEW_GAME_SIMPLIFIED_CHINESE.as_ptr(), new_game_string) } != 0 {
        LanguageType::SimplifiedChinese
    } else if unsafe { string_equals(NEW_GAME_KOREAN.as_ptr(), new_game_string) } != 0 {
        LanguageType::Korean
    } else if unsafe { string_equals(NEW_GAME_JAPANESE.as_ptr(), new_game_string) } != 0 {
        LanguageType::Japanese
    } else {
        LanguageType::Unknown
    }
}

unsafe fn log_language() {
    let desc = match unsafe { DATA.last_determined_language } {
        LanguageType::English => "English\0",
        LanguageType::French => "French\0",
        LanguageType::German => "German\0",
        LanguageType::Greek => "Greek\0",
        LanguageType::Italian => "Italian\0",
        LanguageType::Spanish => "Spanish\0",
        LanguageType::Polish => "Polish\0",
        LanguageType::Portuguese => "Portuguese\0",
        LanguageType::Russian => "Russian\0",
        LanguageType::Swedish => "Swedish\0",
        LanguageType::TraditionalChinese => "Traditional Chinese\0",
        LanguageType::SimplifiedChinese => "Simplified Chinese\0",
        LanguageType::Korean => "Korean\0",
        LanguageType::Japanese => "Japanese\0",
        LanguageType::Czech => "Czech\0",
        _ => "Unknown\0",
    };
    unsafe {
        log_info("Detected language:\0".as_ptr() as *const c_char, desc.as_ptr() as *const c_char, 0);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn locale_determine_language() -> LanguageType {
    unsafe {
        DATA.last_determined_language = determine_language();
        log_language();
        DATA.last_determined_language
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn locale_last_determined_language() -> LanguageType {
    unsafe { DATA.last_determined_language }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn locale_year_before_ad() -> c_int {
    unsafe { (DATA.last_determined_language != LanguageType::English) as c_int }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn locale_translate_money_dn() -> c_int {
    unsafe { (DATA.last_determined_language != LanguageType::Korean) as c_int }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn locale_paragraph_indent() -> c_int {
    unsafe {
        if DATA.last_determined_language == LanguageType::Japanese {
            17
        } else {
            50
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn locale_translate_rank_autosaves() -> c_int {
    unsafe {
        match DATA.last_determined_language {
            LanguageType::English
            | LanguageType::French
            | LanguageType::German
            | LanguageType::Italian
            | LanguageType::Polish
            | LanguageType::Portuguese
            | LanguageType::Spanish
            | LanguageType::Swedish
            | LanguageType::Russian
            | LanguageType::Czech => 1,
            _ => 0,
        }
    }
}
