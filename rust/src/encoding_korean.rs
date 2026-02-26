use std::ffi::{c_char, c_int};

#[repr(C)]
struct KoreanEntry {
    _cp949: u16,
    _utf8: [u8; 3],
}

#[repr(C)]
struct KoreanImageEntry {
    _cp949: u16,
    _image_id: c_int,
}

static _CODEPAGE_TO_UTF8: &[KoreanEntry] = &[];
static _CODEPAGE_TO_IMAGE: &[KoreanImageEntry] = &[];

#[unsafe(no_mangle)]
pub unsafe extern "C" fn encoding_korean_init() {}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn encoding_korean_to_utf8(_input: *const u8, output: *mut c_char, _max_length: c_int) {
    unsafe { *output = 0; }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn encoding_korean_from_utf8(_input: *const c_char, output: *mut u8, _max_length: c_int) {
    unsafe { *output = 0; }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn encoding_korean_cp949_to_image_id(first: u8, second: u8) -> c_int {
    let _cp949 = ((first as u16) << 8) | second as u16;
    0
}
