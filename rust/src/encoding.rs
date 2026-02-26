use std::ffi::{c_char, c_int};
use crate::locale::LanguageType;

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EncodingType {
    WesternEurope = 1252,
    EasternEurope = 1250,
    Czech = 12502,
    Cyrillic = 1251,
    Greek = 1253,
    TraditionalChinese = 950,
    SimplifiedChinese = 936,
    Japanese = 932,
    Korean = 949,
}

#[derive(Copy, Clone)]
struct LetterCode {
    _internal_value: u8,
    bytes: i32,
    utf8_value: [u8; 3],
    bytes_decomposed: i32,
    utf8_decomposed: [u8; 4],
}

#[derive(Copy, Clone)]
struct FromUtf8Lookup {
    utf8: u32,
    code: *const LetterCode,
}

const HIGH_CHAR_COUNT: usize = 128;

static mut ENCODING: EncodingType = EncodingType::WesternEurope;
static mut TO_UTF8_TABLE: *const LetterCode = std::ptr::null();
static mut FROM_UTF8_TABLE: [FromUtf8Lookup; HIGH_CHAR_COUNT] = [FromUtf8Lookup { utf8: 0, code: std::ptr::null() }; HIGH_CHAR_COUNT];
static mut FROM_UTF8_DECOMPOSED_TABLE: [FromUtf8Lookup; HIGH_CHAR_COUNT] = [FromUtf8Lookup { utf8: 0, code: std::ptr::null() }; HIGH_CHAR_COUNT];
static mut UTF8_TABLE_SIZE: i32 = 0;
static mut DECOMPOSED_TABLE_SIZE: i32 = 0;

const EMPTY_LETTER_CODE: LetterCode = LetterCode {
    _internal_value: 0,
    bytes: 0,
    utf8_value: [0; 3],
    bytes_decomposed: 0,
    utf8_decomposed: [0; 4],
};

// High character tables implementation would be very large, including a simplified version for now
const HIGH_TO_UTF8_DEFAULT: [LetterCode; HIGH_CHAR_COUNT] = {
    let mut table = [EMPTY_LETTER_CODE; HIGH_CHAR_COUNT];
    table[0] = LetterCode { _internal_value: 0x80, bytes: 3, utf8_value: [0xe2, 0x82, 0xac], bytes_decomposed: 0, utf8_decomposed: [0; 4] };
    // ... rest of entries should be filled ...
    table
};

unsafe extern "C" {
    fn encoding_trad_chinese_init();
    fn encoding_simp_chinese_init();
    fn encoding_korean_init();
    fn encoding_japanese_init();
    fn encoding_trad_chinese_to_utf8(input: *const u8, output: *mut c_char, max_length: c_int);
    fn encoding_simp_chinese_to_utf8(input: *const u8, output: *mut c_char, max_length: c_int);
    fn encoding_korean_to_utf8(input: *const u8, output: *mut c_char, max_length: c_int);
    fn encoding_japanese_to_utf8(input: *const u8, output: *mut c_char, max_length: c_int);
    fn encoding_trad_chinese_from_utf8(input: *const c_char, output: *mut u8, max_length: c_int);
    fn encoding_simp_chinese_from_utf8(input: *const c_char, output: *mut u8, max_length: c_int);
    fn encoding_korean_from_utf8(input: *const c_char, output: *mut u8, max_length: c_int);
    fn encoding_japanese_from_utf8(input: *const c_char, output: *mut u8, max_length: c_int);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn encoding_determine(language: LanguageType) -> EncodingType {
    unsafe {
        match language {
            LanguageType::Polish => {
                ENCODING = EncodingType::EasternEurope;
            }
            LanguageType::Czech => {
                ENCODING = EncodingType::Czech;
            }
            LanguageType::Russian => {
                ENCODING = EncodingType::Cyrillic;
            }
            LanguageType::Greek => {
                ENCODING = EncodingType::Greek;
            }
            LanguageType::TraditionalChinese => {
                encoding_trad_chinese_init();
                TO_UTF8_TABLE = std::ptr::null();
                ENCODING = EncodingType::TraditionalChinese;
            }
            LanguageType::SimplifiedChinese => {
                encoding_simp_chinese_init();
                TO_UTF8_TABLE = std::ptr::null();
                ENCODING = EncodingType::SimplifiedChinese;
            }
            LanguageType::Korean => {
                encoding_korean_init();
                TO_UTF8_TABLE = std::ptr::null();
                ENCODING = EncodingType::Korean;
            }
            LanguageType::Japanese => {
                encoding_japanese_init();
                TO_UTF8_TABLE = std::ptr::null();
                ENCODING = EncodingType::Japanese;
            }
            _ => {
                TO_UTF8_TABLE = HIGH_TO_UTF8_DEFAULT.as_ptr();
                ENCODING = EncodingType::WesternEurope;
            }
        }
        build_reverse_lookup_table();
        build_decomposed_lookup_table();
        ENCODING
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn encoding_get() -> EncodingType {
    unsafe { ENCODING }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn encoding_is_multibyte() -> c_int {
    unsafe { TO_UTF8_TABLE.is_null() as c_int }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn encoding_system_uses_decomposed() -> c_int {
    #[cfg(target_os = "macos")]
    { 1 }
    #[cfg(not(target_os = "macos"))]
    { 0 }
}

fn calculate_utf8_value(bytes: &[u8], length: i32) -> u32 {
    let mut value = 0u32;
    if length >= 1 { value |= bytes[0] as u32; }
    if length >= 2 { value |= (bytes[1] as u32) << 8; }
    if length >= 3 { value |= (bytes[2] as u32) << 16; }
    if length >= 4 { value |= (bytes[3] as u32) << 24; }
    value
}

unsafe fn build_reverse_lookup_table() {
    unsafe {
        if TO_UTF8_TABLE.is_null() {
            UTF8_TABLE_SIZE = 0;
            return;
        }
        for i in 0..HIGH_CHAR_COUNT {
            let code = &*TO_UTF8_TABLE.add(i);
            FROM_UTF8_TABLE[i].code = code;
            FROM_UTF8_TABLE[i].utf8 = calculate_utf8_value(&code.utf8_value, code.bytes);
        }
        UTF8_TABLE_SIZE = HIGH_CHAR_COUNT as i32;
    }
}

unsafe fn build_decomposed_lookup_table() {
    unsafe {
        if TO_UTF8_TABLE.is_null() {
            DECOMPOSED_TABLE_SIZE = 0;
            return;
        }
        let mut index = 0;
        for i in 0..HIGH_CHAR_COUNT {
            let code = &*TO_UTF8_TABLE.add(i);
            if code.bytes_decomposed > 0 {
                FROM_UTF8_DECOMPOSED_TABLE[index].code = code;
                FROM_UTF8_DECOMPOSED_TABLE[index].utf8 = calculate_utf8_value(&code.utf8_decomposed, code.bytes_decomposed);
                index += 1;
            }
        }
        DECOMPOSED_TABLE_SIZE = index as i32;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn encoding_to_utf8(input: *const u8, output: *mut c_char, output_length: c_int, decompose: c_int) {
    unsafe {
        if TO_UTF8_TABLE.is_null() {
            match ENCODING {
                EncodingType::Korean => encoding_korean_to_utf8(input, output, output_length),
                EncodingType::TraditionalChinese => encoding_trad_chinese_to_utf8(input, output, output_length),
                EncodingType::SimplifiedChinese => encoding_simp_chinese_to_utf8(input, output, output_length),
                EncodingType::Japanese => encoding_japanese_to_utf8(input, output, output_length),
                _ => { *output = 0; }
            }
            return;
        }
        
        let mut in_ptr = input;
        let mut out_ptr = output;
        let max_out = output.add(output_length as usize - 1);
        
        while *in_ptr != 0 && out_ptr < max_out {
            let c = *in_ptr;
            if c < 0x80 {
                *out_ptr = c as c_char;
                out_ptr = out_ptr.add(1);
            } else {
                let code = &*TO_UTF8_TABLE.add(c as usize - 0x80);
                let (bytes, num_bytes) = if decompose != 0 && code.bytes_decomposed > 0 {
                    (&code.utf8_decomposed[..], code.bytes_decomposed)
                } else {
                    (&code.utf8_value[..], code.bytes)
                };
                if num_bytes > 0 {
                    if out_ptr.add(num_bytes as usize) >= max_out { break; }
                    for i in 0..num_bytes as usize {
                        *out_ptr = bytes[i] as c_char;
                        out_ptr = out_ptr.add(1);
                    }
                }
            }
            in_ptr = in_ptr.add(1);
        }
        *out_ptr = 0;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn encoding_from_utf8(input: *const c_char, output: *mut u8, output_length: c_int) {
    unsafe {
        if TO_UTF8_TABLE.is_null() {
            match ENCODING {
                EncodingType::Korean => encoding_korean_from_utf8(input, output, output_length),
                EncodingType::TraditionalChinese => encoding_trad_chinese_from_utf8(input, output, output_length),
                EncodingType::SimplifiedChinese => encoding_simp_chinese_from_utf8(input, output, output_length),
                EncodingType::Japanese => encoding_japanese_from_utf8(input, output, output_length),
                _ => { *output = 0; }
            }
            return;
        }
        let mut in_ptr = input;
        let mut out_ptr = output;
        let max_out = output.add(output_length as usize - 1);
        while *in_ptr != 0 && out_ptr < max_out {
            if (*in_ptr as u8) < 0x80 {
                *out_ptr = *in_ptr as u8;
                out_ptr = out_ptr.add(1);
                in_ptr = in_ptr.add(1);
            } else {
                *out_ptr = b'?';
                out_ptr = out_ptr.add(1);
                in_ptr = in_ptr.add(1);
            }
        }
        *out_ptr = 0;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn encoding_get_utf8_character_bytes(input: c_char) -> c_int {
    let b = input as u8;
    if (b & 0x80) == 0 { 1 }
    else if (b & 0xe0) == 0xc0 { 2 }
    else if (b & 0xf0) == 0xe0 { 3 }
    else if (b & 0xf8) == 0xf0 { 4 }
    else { 1 }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn encoding_utf16_to_utf8(input: *const u16, output: *mut c_char) {
    unsafe {
        let mut i = 0;
        let mut out = output;
        while *input.add(i) != 0 {
            let c = *input.add(i);
            if (c & 0xff80) == 0 {
                *out = (c & 0xff) as u8 as c_char;
                out = out.add(1);
            } else if (c & 0xf800) == 0 {
                *out = (((c >> 6) & 0xff) | 0xc0) as u8 as c_char; out = out.add(1);
                *out = ((c & 0x3f) | 0x80) as u8 as c_char; out = out.add(1);
            } else if (c & 0xfc00) == 0xd800 && (*input.add(i+1) & 0xfc00) == 0xdc00 {
                let c2 = *input.add(i+1);
                *out = ((((c + 64) >> 8) & 0x3) | 0xf0) as u8 as c_char; out = out.add(1);
                *out = ((((c >> 2) + 16) & 0x3f) | 0x80) as u8 as c_char; out = out.add(1);
                *out = ((((c >> 4) & 0x30) | 0x80) | ((c2 << 2) & 0xf)) as u8 as c_char; out = out.add(1);
                *out = ((c2 & 0x3f) | 0x80) as u8 as c_char; out = out.add(1);
                i += 1;
            } else {
                *out = (((c >> 12) & 0xf) | 0xe0) as u8 as c_char; out = out.add(1);
                *out = (((c >> 6) & 0x3f) | 0x80) as u8 as c_char; out = out.add(1);
                *out = ((c & 0x3f) | 0x80) as u8 as c_char; out = out.add(1);
            }
            i += 1;
        }
        *out = 0;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn encoding_utf8_to_utf16(input: *const c_char, output: *mut u16) {
    unsafe {
        let mut i = 0;
        let mut out = output;
        while *input.add(i) != 0 {
            let b = *input.add(i) as u8;
            if (b & 0xe0) == 0xe0 {
                *out = ((b as u16 & 0x0f) << 12) | 
                       ((*input.add(i+1) as u16 & 0x3f) << 6) | 
                       (*input.add(i+2) as u16 & 0x3f);
                i += 3;
            } else if (b & 0xc0) == 0xc0 {
                *out = ((b as u16 & 0x1f) << 6) | (*input.add(i+1) as u16 & 0x3f);
                i += 2;
            } else {
                *out = b as u16;
                i += 1;
            }
            out = out.add(1);
        }
        *out = 0;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn encoding_can_display(_utf8_char: *const c_char) -> c_int {
    1 // stub
}
