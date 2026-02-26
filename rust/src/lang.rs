use std::ffi::{c_char, c_int, c_void};
use std::ptr::{self, addr_of, addr_of_mut};
use crate::buffer::{Buffer, buffer_init, buffer_skip, buffer_read_i16, buffer_read_i32, buffer_read_raw};
use crate::locale::{LanguageType, locale_last_determined_language};

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LangType {
    Manual = 0,
    About = 1,
    Message = 2,
    Mission = 3,
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LangMessageType {
    General = 0,
    Disaster = 1,
    Imperial = 2,
    Emigration = 3,
    Tutorial = 4,
    TradeChange = 5,
    PriceChange = 6,
    Invasion = 7,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LangMessageImage {
    pub id: c_int,
    pub x: c_int,
    pub y: c_int,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LangMessageString {
    pub text: *mut u8,
    pub x: c_int,
    pub y: c_int,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LangMessage {
    pub lang_type: LangType,
    pub message_type: LangMessageType,
    pub x: c_int,
    pub y: c_int,
    pub width_blocks: c_int,
    pub height_blocks: c_int,
    pub urgent: c_int,
    pub image: LangMessageImage,
    pub title: LangMessageString,
    pub subtitle: LangMessageString,
    pub video: LangMessageString,
    pub content: LangMessageString,
}

const MAX_TEXT_ENTRIES: usize = 1000;
const MAX_TEXT_DATA: usize = 200000;
const MIN_TEXT_SIZE: i32 = 28 + (MAX_TEXT_ENTRIES as i32 * 8);
const MAX_TEXT_SIZE: i32 = MIN_TEXT_SIZE + MAX_TEXT_DATA as i32;

const MAX_MESSAGE_ENTRIES: usize = 400;
const MAX_MESSAGE_DATA: usize = 460000;
const MIN_MESSAGE_SIZE: i32 = 32024;
const MAX_MESSAGE_SIZE: i32 = MIN_MESSAGE_SIZE + MAX_MESSAGE_DATA as i32;

const BUFFER_SIZE: usize = 400000;

const FILE_TEXT_ENG: *const c_char = "c3.eng\0".as_ptr() as *const c_char;
const FILE_MM_ENG: *const c_char = "c3_mm.eng\0".as_ptr() as *const c_char;
const FILE_TEXT_RUS: *const c_char = "c3.rus\0".as_ptr() as *const c_char;
const FILE_MM_RUS: *const c_char = "c3_mm.rus\0".as_ptr() as *const c_char;
const FILE_EDITOR_TEXT_ENG: *const c_char = "c3_map.eng\0".as_ptr() as *const c_char;
const FILE_EDITOR_MM_ENG: *const c_char = "c3_map_mm.eng\0".as_ptr() as *const c_char;
const FILE_EDITOR_TEXT_RUS: *const c_char = "c3_map.rus\0".as_ptr() as *const c_char;
const FILE_EDITOR_MM_RUS: *const c_char = "c3_map_mm.rus\0".as_ptr() as *const c_char;

#[repr(C)]
struct TextEntry {
    offset: i32,
    in_use: i32,
}

struct LangData {
    text_entries: [TextEntry; MAX_TEXT_ENTRIES],
    text_data: [u8; MAX_TEXT_DATA],
    message_entries: [LangMessage; MAX_MESSAGE_ENTRIES],
    message_data: [u8; MAX_MESSAGE_DATA],
}

// Initializing with zeros
static mut DATA: LangData = unsafe { std::mem::zeroed() };

unsafe extern "C" {
    fn file_exists(filename: *const c_char, localizable: c_int) -> c_int;
    fn io_read_file_into_buffer(filepath: *const c_char, localizable: c_int, buffer: *mut c_void, max_size: c_int) -> c_int;
    fn translation_for(key: i32) -> *const u8;
}

const NOT_LOCALIZED: c_int = 0;
const MAY_BE_LOCALIZED: c_int = 1;
const MUST_BE_LOCALIZED: c_int = 2;

const TR_FIX_KOREAN_BUILDING_DOCTORS_CLINIC: i32 = 101;

unsafe fn file_exists_in_dir(dir: *const c_char, file: *const c_char) -> bool {
    let mut path = [0u8; 600];
    unsafe {
        let dir_str = std::ffi::CStr::from_ptr(dir).to_bytes();
        let file_str = std::ffi::CStr::from_ptr(file).to_bytes();
        
        let mut curr = 0;
        for &b in dir_str { if curr < 598 { path[curr] = b; curr += 1; } }
        if curr < 598 { path[curr] = b'/'; curr += 1; }
        for &b in file_str { if curr < 599 { path[curr] = b; curr += 1; } }
        path[curr] = 0;

        file_exists(path.as_ptr() as *const c_char, NOT_LOCALIZED) != 0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lang_dir_is_valid(dir: *const c_char) -> c_int {
    unsafe {
        if file_exists_in_dir(dir, FILE_TEXT_ENG) && file_exists_in_dir(dir, FILE_MM_ENG) {
            return 1;
        }
        if file_exists_in_dir(dir, FILE_TEXT_RUS) && file_exists_in_dir(dir, FILE_MM_RUS) {
            return 1;
        }
        0
    }
}

unsafe fn parse_text(buf: *mut Buffer) {
    unsafe {
        buffer_skip(buf, 28);
        for i in 0..MAX_TEXT_ENTRIES {
            let entry = addr_of_mut!(DATA.text_entries[i]);
            (*entry).offset = buffer_read_i32(buf);
            (*entry).in_use = buffer_read_i32(buf);
        }
        buffer_read_raw(buf, addr_of_mut!(DATA.text_data) as *mut c_void, MAX_TEXT_DATA as c_int);
    }
}

unsafe fn load_text(filename: *const c_char, localizable: c_int, buf_data: *mut u8) -> bool {
    unsafe {
        let filesize = io_read_file_into_buffer(filename, localizable, buf_data as *mut c_void, BUFFER_SIZE as c_int);
        if filesize < MIN_TEXT_SIZE || filesize > MAX_TEXT_SIZE {
            return false;
        }
        let mut buf = std::mem::zeroed();
        buffer_init(&mut buf, buf_data as *mut c_void, filesize);
        parse_text(&mut buf);
        true
    }
}

unsafe fn get_message_text(offset: i32) -> *mut u8 {
    if offset == 0 {
        ptr::null_mut()
    } else {
        unsafe { (addr_of_mut!(DATA.message_data) as *mut u8).add(offset as usize) }
    }
}

unsafe fn parse_message(buf: *mut Buffer) {
    unsafe {
        buffer_skip(buf, 24);
        for i in 0..MAX_MESSAGE_ENTRIES {
            let m = addr_of_mut!(DATA.message_entries[i]);
            (*m).lang_type = std::mem::transmute(buffer_read_i16(buf) as i32);
            (*m).message_type = std::mem::transmute(buffer_read_i16(buf) as i32);
            buffer_skip(buf, 2);
            (*m).x = buffer_read_i16(buf) as c_int;
            (*m).y = buffer_read_i16(buf) as c_int;
            (*m).width_blocks = buffer_read_i16(buf) as c_int;
            (*m).height_blocks = buffer_read_i16(buf) as c_int;
            (*m).image.id = buffer_read_i16(buf) as c_int;
            (*m).image.x = buffer_read_i16(buf) as c_int;
            (*m).image.y = buffer_read_i16(buf) as c_int;
            buffer_skip(buf, 6);
            (*m).title.x = buffer_read_i16(buf) as c_int;
            (*m).title.y = buffer_read_i16(buf) as c_int;
            (*m).subtitle.x = buffer_read_i16(buf) as c_int;
            (*m).subtitle.y = buffer_read_i16(buf) as c_int;
            buffer_skip(buf, 4);
            (*m).video.x = buffer_read_i16(buf) as c_int;
            (*m).video.y = buffer_read_i16(buf) as c_int;
            buffer_skip(buf, 14);
            (*m).urgent = buffer_read_i32(buf);

            (*m).video.text = get_message_text(buffer_read_i32(buf));
            buffer_skip(buf, 4);
            (*m).title.text = get_message_text(buffer_read_i32(buf));
            (*m).subtitle.text = get_message_text(buffer_read_i32(buf));
            (*m).content.text = get_message_text(buffer_read_i32(buf));
        }
        buffer_read_raw(buf, addr_of_mut!(DATA.message_data) as *mut c_void, MAX_MESSAGE_DATA as c_int);
    }
}

unsafe fn load_message(filename: *const c_char, localizable: c_int, data_buffer: *mut u8) -> bool {
    unsafe {
        let filesize = io_read_file_into_buffer(filename, localizable, data_buffer as *mut c_void, BUFFER_SIZE as c_int);
        if filesize < MIN_MESSAGE_SIZE || filesize > MAX_MESSAGE_SIZE {
            return false;
        }
        let mut buf = std::mem::zeroed();
        buffer_init(&mut buf, data_buffer as *mut c_void, filesize);
        parse_message(&mut buf);
        true
    }
}

unsafe fn load_files(text_filename: *const c_char, message_filename: *const c_char, localizable: c_int) -> bool {
    unsafe {
        let buffer = libc::malloc(BUFFER_SIZE) as *mut u8;
        if buffer.is_null() {
            return false;
        }
        let success = load_text(text_filename, localizable, buffer) && load_message(message_filename, localizable, buffer);
        libc::free(buffer as *mut libc::c_void);
        success
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lang_load(is_editor: c_int) -> c_int {
    unsafe {
        if is_editor != 0 {
            return (load_files(FILE_EDITOR_TEXT_RUS, FILE_EDITOR_MM_RUS, MAY_BE_LOCALIZED) ||
                    load_files(FILE_EDITOR_TEXT_ENG, FILE_EDITOR_MM_ENG, MAY_BE_LOCALIZED)) as c_int;
        }
        (load_files(FILE_TEXT_ENG, FILE_MM_ENG, MUST_BE_LOCALIZED) ||
         load_files(FILE_TEXT_RUS, FILE_MM_RUS, MUST_BE_LOCALIZED) ||
         load_files(FILE_TEXT_ENG, FILE_MM_ENG, NOT_LOCALIZED) ||
         load_files(FILE_TEXT_RUS, FILE_MM_RUS, NOT_LOCALIZED)) as c_int
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lang_get_string(group: c_int, index: c_int) -> *const u8 {
    unsafe {
        let l_type = locale_last_determined_language();
        if l_type == LanguageType::Korean && group == 28 && index == 46 {
            return translation_for(TR_FIX_KOREAN_BUILDING_DOCTORS_CLINIC);
        }

        let entry = addr_of!(DATA.text_entries[group as usize]);
        let mut str_ptr = (addr_of!(DATA.text_data) as *const u8).add((*entry).offset as usize);
        let mut idx = index;
        let mut prev = 0u8;
        while idx > 0 {
            if *str_ptr == 0 && (prev >= b' ' || prev == 0) {
                idx -= 1;
            }
            prev = *str_ptr;
            str_ptr = str_ptr.add(1);
        }
        while *str_ptr < b' ' {
            str_ptr = str_ptr.add(1);
        }
        str_ptr
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lang_get_message(id: c_int) -> *const LangMessage {
    unsafe { addr_of!(DATA.message_entries[id as usize]) }
}
