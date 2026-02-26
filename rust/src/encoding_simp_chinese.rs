use std::ffi::{c_char, c_int};

#[repr(C)]
struct ChineseEntry {
    _cp936: u16,
    _utf8: [u8; 3],
}

#[repr(C)]
struct ChineseImageEntry {
    _cp936: u16,
    _image_id: c_int,
}

static _CODEPAGE_TO_UTF8: &[ChineseEntry] = &[];
static _CODEPAGE_TO_IMAGE: &[ChineseImageEntry] = &[];

#[unsafe(no_mangle)]
pub unsafe extern "C" fn encoding_simp_chinese_init() {}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn encoding_simp_chinese_to_utf8(_input: *const u8, output: *mut c_char, _max_length: c_int) {
    unsafe { *output = 0; }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn encoding_simp_chinese_from_utf8(_input: *const c_char, output: *mut u8, _max_length: c_int) {
    unsafe { *output = 0; }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn encoding_simp_chinese_gb2312_to_image_id(first: u8, second: u8) -> c_int {
    let _cp936 = ((first as u16) << 8) | second as u16;
    0
}
