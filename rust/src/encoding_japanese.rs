use std::ffi::{c_char, c_int};

#[repr(C)]
struct JapaneseEntry {
    _cp932: u16,
    _utf8: [u8; 3],
}

#[repr(C)]
struct JapaneseImageEntry {
    _cp932: u16,
    _image_id: c_int,
}

static _CODEPAGE_TO_UTF8: &[JapaneseEntry] = &[];
static _CODEPAGE_TO_IMAGE: &[JapaneseImageEntry] = &[];

#[unsafe(no_mangle)]
pub unsafe extern "C" fn encoding_japanese_init() {}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn encoding_japanese_to_utf8(_input: *const u8, output: *mut c_char, _output_length: c_int) {
    unsafe {
        *output = 0;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn encoding_japanese_from_utf8(_input: *const c_char, output: *mut u8, _output_length: c_int) {
    unsafe {
        *output = 0;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn encoding_japanese_sjis_to_image_id(first: u8, second: u8) -> c_int {
    let _sjis = ((first as u16) << 8) | second as u16;
    0
}
