use std::ffi::{c_char, c_int, c_void};
use std::ptr::{self, addr_of, addr_of_mut};

// PKWare constants
const PK_SUCCESS: c_int = 0;
const PK_INVALID_WINDOWSIZE: c_int = 1;
const PK_LITERAL_ENCODING_UNSUPPORTED: c_int = 2;
const PK_TOO_FEW_INPUT_BYTES: c_int = 3;
const PK_ERROR_DECODING: c_int = 4;
const PK_ERROR_VALUE: i32 = 774;
const PK_EOF: i32 = 773;

struct PkToken {
    stop: bool,
    input_data: *const u8,
    input_ptr: usize,
    input_length: usize,
    output_data: *mut u8,
    output_ptr: usize,
    output_length: usize,
}

type PkInputFunc = fn(&mut [u8], &mut PkToken) -> usize;
type PkOutputFunc = fn(&[u8], &mut PkToken);

struct PkCompBuffer {
    dictionary_size: usize,
    window_size: u32,
    copy_offset_extra_mask: u32,
    current_output_bits_used: u32,
    // Increased size to handle matching past the end of the current block
    input_data: Box<[u8; 8708 + 516]>,
    output_data: Box<[u8; 2050]>,
    output_ptr: usize,
    analyze_offset_table: Box<[u16; 2304]>,
    analyze_index: Box<[u16; 8708]>,
    long_matcher: Box<[i16; 518]>,
    codeword_values: Box<[u16; 774]>,
    codeword_bits: Box<[u8; 774]>,
}

struct PkDecompBuffer {
    window_size: u32,
    dictionary_size: u32,
    current_input_byte: u16,
    current_input_bits_available: u32,
    input_buffer_ptr: usize,
    input_buffer_end: usize,
    output_buffer_ptr: usize,
    input_buffer: Box<[u8; 2048]>,
    output_buffer: Box<[u8; 8708 + 516]>,
    copy_offset_jump_table: Box<[u8; 256]>,
    copy_length_jump_table: Box<[u8; 256]>,
}

#[derive(Copy, Clone)]
struct PkCopyLengthOffset {
    length: usize,
    offset: u16,
}

static PK_COPY_OFFSET_BITS: [u8; 64] = [
    2, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6, 6, 6, 6,
    6, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
    7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
];

static PK_COPY_OFFSET_CODE: [u8; 64] = [
    0x03, 0x0D, 0x05, 0x19, 0x09, 0x11, 0x01, 0x3E,
    0x1E, 0x2E, 0x0E, 0x36, 0x16, 0x26, 0x06, 0x3A,
    0x1A, 0x2A, 0x0A, 0x32, 0x12, 0x22, 0x42, 0x02,
    0x7C, 0x3C, 0x5C, 0x1C, 0x6C, 0x2C, 0x4C, 0x0C,
    0x74, 0x34, 0x54, 0x14, 0x64, 0x24, 0x44, 0x04,
    0x78, 0x38, 0x58, 0x18, 0x68, 0x28, 0x48, 0x08,
    0xF0, 0x70, 0xB0, 0x30, 0xD0, 0x50, 0x90, 0x10,
    0xE0, 0x60, 0xA0, 0x20, 0xC0, 0x40, 0x80, 0x00,
];

static PK_COPY_LENGTH_BASE_BITS: [u8; 16] = [
    3, 2, 3, 3, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 7, 7,
];

static PK_COPY_LENGTH_BASE_VALUE: [u16; 16] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
    0x08, 0x0A, 0x0E, 0x16, 0x26, 0x46, 0x86, 0x106,
];

static PK_COPY_LENGTH_BASE_CODE: [u8; 16] = [
    0x05, 0x03, 0x01, 0x06, 0x0A, 0x02, 0x0C, 0x14,
    0x04, 0x18, 0x08, 0x30, 0x10, 0x20, 0x40, 0x00,
];

static PK_COPY_LENGTH_EXTRA_BITS: [u8; 16] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8,
];

unsafe extern "C" {
    fn log_error(msg: *const c_char, param_str: *const c_char, param_int: c_int);
}

fn pk_implode_fill_input_buffer(buf: &mut PkCompBuffer, mut bytes_to_read: usize, input_func: PkInputFunc, token: &mut PkToken) -> usize {
    let mut used = 0;
    while bytes_to_read > 0 {
        let start = buf.dictionary_size + 516 + used;
        let read = input_func(&mut buf.input_data[start..start + bytes_to_read], token);
        if read == 0 { break; }
        used += read;
        bytes_to_read -= read;
    }
    used
}

fn pk_implode_flush_full_buffer(buf: &mut PkCompBuffer, output_func: PkOutputFunc, token: &mut PkToken) {
    output_func(&buf.output_data[0..2048], token);
    let new_first_byte = buf.output_data[2048];
    let last_byte = buf.output_data[buf.output_ptr];
    buf.output_ptr -= 2048;
    buf.output_data[0..2050].fill(0);
    if buf.output_ptr > 0 {
        buf.output_data[0] = new_first_byte;
    }
    if buf.current_output_bits_used > 0 {
        buf.output_data[buf.output_ptr] = last_byte;
    }
}

fn pk_implode_write_bits(buf: &mut PkCompBuffer, mut num_bits: u32, mut value: u32, output_func: PkOutputFunc, token: &mut PkToken) {
    if num_bits > 8 {
        pk_implode_write_bits(buf, 8, value, output_func, token);
        num_bits -= 8;
        value >>= 8;
    }
    let current_bits_used = buf.current_output_bits_used;
    buf.output_data[buf.output_ptr] |= (value << current_bits_used) as u8;
    buf.current_output_bits_used += num_bits;
    if buf.current_output_bits_used == 8 {
        buf.output_ptr += 1;
        buf.current_output_bits_used = 0;
    } else if buf.current_output_bits_used > 8 {
        buf.output_ptr += 1;
        buf.output_data[buf.output_ptr] = (value >> (8 - current_bits_used)) as u8;
        buf.current_output_bits_used -= 8;
    }
    if buf.output_ptr >= 2048 {
        pk_implode_flush_full_buffer(buf, output_func, token);
    }
}

fn pk_implode_write_copy_length_offset(buf: &mut PkCompBuffer, copy: PkCopyLengthOffset, output_func: PkOutputFunc, token: &mut PkToken) {
    let code_idx = copy.length + 254;
    pk_implode_write_bits(buf, buf.codeword_bits[code_idx] as u32, buf.codeword_values[code_idx] as u32, output_func, token);

    if copy.length == 2 {
        let idx = (copy.offset >> 2) as usize;
        pk_implode_write_bits(buf, PK_COPY_OFFSET_BITS[idx] as u32, PK_COPY_OFFSET_CODE[idx] as u32, output_func, token);
        pk_implode_write_bits(buf, 2, (copy.offset & 3) as u32, output_func, token);
    } else {
        let idx = (copy.offset >> buf.window_size) as usize;
        pk_implode_write_bits(buf, PK_COPY_OFFSET_BITS[idx] as u32, PK_COPY_OFFSET_CODE[idx] as u32, output_func, token);
        pk_implode_write_bits(buf, buf.window_size, (copy.offset & buf.copy_offset_extra_mask as u16) as u32, output_func, token);
    }
}

fn pk_implode_determine_copy(buf: &mut PkCompBuffer, input_index: usize, input_end: usize, copy: &mut PkCopyLengthOffset) {
    let input_ptr_val = &buf.input_data[input_index..];
    let hash_value = 4 * input_ptr_val[0] as usize + 5 * input_ptr_val[1] as usize;
    let mut hash_analyze_index = buf.analyze_offset_table[hash_value] as usize;
    let min_match_index = input_index as i32 - buf.dictionary_size as i32 + 1;
    
    while (buf.analyze_index[hash_analyze_index] as i32) < min_match_index {
        hash_analyze_index += 1;
    }
    buf.analyze_offset_table[hash_value] = hash_analyze_index as u16;

    let mut max_matched_bytes = 1;
    let mut current_hash_idx = hash_analyze_index;
    let mut match_idx = buf.analyze_index[current_hash_idx] as usize;
    
    if input_index - 1 <= match_idx {
        copy.length = 0;
        return;
    }

    loop {
        if buf.input_data[match_idx + max_matched_bytes - 1] == buf.input_data[input_index + max_matched_bytes - 1] &&
           buf.input_data[match_idx] == buf.input_data[input_index] {
            let mut matched_bytes = 2;
            let max_possible = std::cmp::min(516, input_end - input_index);
            while matched_bytes < max_possible && buf.input_data[match_idx + matched_bytes] == buf.input_data[input_index + matched_bytes] {
                matched_bytes += 1;
            }
            if matched_bytes >= max_matched_bytes {
                copy.offset = (input_index - match_idx - 1) as u16;
                max_matched_bytes = matched_bytes;
                if matched_bytes > 10 { break; }
            }
        }
        current_hash_idx += 1;
        match_idx = buf.analyze_index[current_hash_idx] as usize;
        if input_index - 1 <= match_idx {
            copy.length = if max_matched_bytes < 2 { 0 } else { max_matched_bytes };
            return;
        }
    }
    if max_matched_bytes == 516 {
        copy.length = 516;
        copy.offset -= 1;
        return;
    }
    if (buf.analyze_index[current_hash_idx + 1] as usize) >= input_index - 1 {
        copy.length = max_matched_bytes;
        return;
    }
    copy.length = max_matched_bytes;
}

fn pk_implode_next_copy_is_better(buf: &mut PkCompBuffer, input_ptr: usize, input_end: usize, current_copy: &PkCopyLengthOffset) -> bool {
    let mut next_copy = PkCopyLengthOffset { length: 0, offset: 0 };
    pk_implode_determine_copy(buf, input_ptr + 1, input_end, &mut next_copy);
    if current_copy.length >= next_copy.length {
        return false;
    }
    if current_copy.length + 1 == next_copy.length && current_copy.offset <= 128 {
        return false;
    }
    true
}

fn pk_implode_analyze_input(buf: &mut PkCompBuffer, input_start: usize, input_end: usize) {
    buf.analyze_offset_table.fill(0);
    // Don't analyze the very last byte to avoid hash access overflow
    let limit = if input_end > 0 { input_end - 1 } else { 0 };
    for index in input_start..limit {
        let hash = 4 * buf.input_data[index] as usize + 5 * buf.input_data[index+1] as usize;
        buf.analyze_offset_table[hash] += 1;
    }
    let mut running_total = 0;
    for i in 0..2304 {
        running_total += buf.analyze_offset_table[i];
        buf.analyze_offset_table[i] = running_total;
    }
    for index in (input_start..limit).rev() {
        let hash = 4 * buf.input_data[index] as usize + 5 * buf.input_data[index+1] as usize;
        buf.analyze_offset_table[hash] -= 1;
        let val = buf.analyze_offset_table[hash];
        buf.analyze_index[val as usize] = index as u16;
    }
}

fn pk_implode_data(buf: &mut PkCompBuffer, input_func: PkInputFunc, output_func: PkOutputFunc, token: &mut PkToken) {
    let mut eof = false;
    let mut has_leftover_data = 0;
    buf.output_data[0] = 0; // no literal encoding
    buf.output_data[1] = buf.window_size as u8;
    buf.output_ptr = 2;
    let mut input_ptr = buf.dictionary_size + 516;
    buf.output_data[2..2050].fill(0);
    buf.current_output_bits_used = 0;

    while !eof {
        let bytes_read = pk_implode_fill_input_buffer(buf, 4096, input_func, token);
        if bytes_read != 4096 {
            eof = true;
            if bytes_read == 0 && has_leftover_data == 0 { break; }
        }
        let mut input_end = buf.dictionary_size + bytes_read;
        if eof { input_end += 516; }

        if has_leftover_data == 0 {
            pk_implode_analyze_input(buf, input_ptr, input_end + 1);
            has_leftover_data += 1;
            if buf.dictionary_size != 4096 { has_leftover_data += 1; }
        } else if has_leftover_data == 1 {
            pk_implode_analyze_input(buf, input_ptr - buf.dictionary_size + 516, input_end + 1);
            has_leftover_data += 1;
        } else if has_leftover_data == 2 {
            pk_implode_analyze_input(buf, input_ptr - buf.dictionary_size, input_end + 1);
        }

        while input_ptr < input_end {
            let mut copy = PkCopyLengthOffset { length: 0, offset: 0 };
            pk_implode_determine_copy(buf, input_ptr, input_end, &mut copy);
            
            let mut write_literal = false;
            let mut write_copy = false;

            if copy.length == 0 {
                write_literal = true;
            } else if copy.length == 2 && copy.offset >= 256 {
                write_literal = true;
            } else if eof && input_ptr + copy.length > input_end {
                copy.length = input_end - input_ptr;
                if copy.length > 2 || (copy.length == 2 && copy.offset < 256) {
                    write_copy = true;
                } else {
                    write_literal = true;
                }
            } else if copy.length >= 8 || input_ptr + 1 >= input_end {
                write_copy = true;
            } else if pk_implode_next_copy_is_better(buf, input_ptr, input_end, &copy) {
                write_literal = true;
            } else {
                write_copy = true;
            }

            if write_copy {
                pk_implode_write_copy_length_offset(buf, copy, output_func, token);
                input_ptr += copy.length;
            } else if write_literal {
                let literal = buf.input_data[input_ptr];
                let bits = buf.codeword_bits[literal as usize] as u32;
                let val = buf.codeword_values[literal as usize] as u32;
                pk_implode_write_bits(buf, bits, val, output_func, token);
                input_ptr += 1;
            }
        }
        if !eof {
            input_ptr -= 4096;
            let len = buf.dictionary_size + 516;
            unsafe {
                ptr::copy(buf.input_data.as_ptr().add(4096), buf.input_data.as_mut_ptr(), len);
            }
        }
    }
    pk_implode_write_bits(buf, buf.codeword_bits[PK_EOF as usize] as u32, buf.codeword_values[PK_EOF as usize] as u32, output_func, token);
    if buf.current_output_bits_used > 0 { buf.output_ptr += 1; }
    output_func(&buf.output_data[0..buf.output_ptr], token);
}

fn pk_explode_construct_jump_table(size: usize, bits: &[u8], codes: &[u8], jump: &mut [u8; 256]) {
    for i in (0..size).rev() {
        let bit = bits[i];
        let mut code = codes[i] as usize;
        loop {
            jump[code] = i as u8;
            code += 1 << bit;
            if code >= 0x100 { break; }
        }
    }
}

fn pk_explode_set_bits_used(buf: &mut PkDecompBuffer, num_bits: u32, input_func: PkInputFunc, token: &mut PkToken) -> bool {
    if buf.current_input_bits_available >= num_bits {
        buf.current_input_bits_available -= num_bits;
        buf.current_input_byte >>= num_bits;
        return false;
    }
    buf.current_input_byte >>= buf.current_input_bits_available;
    if buf.input_buffer_ptr == buf.input_buffer_end {
        buf.input_buffer_ptr = 0;
        buf.input_buffer_end = input_func(&mut *buf.input_buffer, token);
        if buf.input_buffer_end == 0 { return true; }
    }
    buf.current_input_byte |= (buf.input_buffer[buf.input_buffer_ptr] as u16) << 8;
    buf.input_buffer_ptr += 1;
    buf.current_input_byte >>= num_bits - buf.current_input_bits_available;
    buf.current_input_bits_available += 8 - num_bits;
    false
}

fn pk_explode_decode_next_token(buf: &mut PkDecompBuffer, input_func: PkInputFunc, token: &mut PkToken) -> i32 {
    if (buf.current_input_byte & 1) != 0 {
        if pk_explode_set_bits_used(buf, 1, input_func, token) { return PK_ERROR_VALUE; }
        let idx = buf.copy_length_jump_table[(buf.current_input_byte & 0xff) as usize] as usize;
        if pk_explode_set_bits_used(buf, PK_COPY_LENGTH_BASE_BITS[idx] as u32, input_func, token) { return PK_ERROR_VALUE; }
        let extra = PK_COPY_LENGTH_EXTRA_BITS[idx] as u32;
        let mut val = idx as i32;
        if extra > 0 {
            let extra_val = (buf.current_input_byte & ((1 << extra) - 1)) as i32;
            if pk_explode_set_bits_used(buf, extra, input_func, token) && idx + extra_val as usize != 270 {
                return PK_ERROR_VALUE;
            }
            val = PK_COPY_LENGTH_BASE_VALUE[idx] as i32 + extra_val;
        }
        val + 256
    } else {
        if pk_explode_set_bits_used(buf, 1, input_func, token) { return PK_ERROR_VALUE; }
        let res = (buf.current_input_byte & 0xff) as i32;
        if pk_explode_set_bits_used(buf, 8, input_func, token) { return PK_ERROR_VALUE; }
        res
    }
}

fn pk_explode_get_copy_offset(buf: &mut PkDecompBuffer, length: usize, input_func: PkInputFunc, token: &mut PkToken) -> u32 {
    let idx = buf.copy_offset_jump_table[(buf.current_input_byte & 0xff) as usize] as usize;
    if pk_explode_set_bits_used(buf, PK_COPY_OFFSET_BITS[idx] as u32, input_func, token) { return 0; }
    let offset = if length == 2 {
        let off = (buf.current_input_byte & 3) | ((idx as u16) << 2);
        if pk_explode_set_bits_used(buf, 2, input_func, token) { return 0; }
        off
    } else {
        let off = (buf.current_input_byte & buf.dictionary_size as u16) | ((idx as u16) << buf.window_size);
        if pk_explode_set_bits_used(buf, buf.window_size, input_func, token) { return 0; }
        off
    };
    offset as u32 + 1
}

fn pk_explode_data(buf: &mut PkDecompBuffer, input_func: PkInputFunc, output_func: PkOutputFunc, token: &mut PkToken) -> i32 {
    buf.output_buffer_ptr = 4096;
    loop {
        let res = pk_explode_decode_next_token(buf, input_func, token);
        if res >= PK_ERROR_VALUE - 1 { return res; }
        if res >= 256 {
            let mut length = (res - 254) as usize;
            let offset = pk_explode_get_copy_offset(buf, length, input_func, token) as usize;
            if offset == 0 { return PK_ERROR_VALUE; }
            let mut src_idx = buf.output_buffer_ptr - offset;
            while length > 0 {
                buf.output_buffer[buf.output_buffer_ptr] = buf.output_buffer[src_idx];
                buf.output_buffer_ptr += 1;
                src_idx += 1;
                length -= 1;
            }
        } else {
            buf.output_buffer[buf.output_buffer_ptr] = res as u8;
            buf.output_buffer_ptr += 1;
        }
        if buf.output_buffer_ptr >= 8192 {
            output_func(&buf.output_buffer[4096..8192], token);
            unsafe {
                ptr::copy(buf.output_buffer.as_ptr().add(4096), buf.output_buffer.as_mut_ptr(), buf.output_buffer_ptr - 4096);
            }
            buf.output_buffer_ptr -= 4096;
        }
    }
}

// Zip input/output functions
fn zip_input_func(buffer: &mut [u8], token: &mut PkToken) -> usize {
    if token.stop || token.input_ptr >= token.input_length { return 0; }
    let mut len = buffer.len();
    if token.input_length - token.input_ptr < len { len = token.input_length - token.input_ptr; }
    unsafe {
        ptr::copy_nonoverlapping(token.input_data.add(token.input_ptr), buffer.as_mut_ptr(), len);
    }
    token.input_ptr += len;
    len
}

fn zip_output_func(buffer: &[u8], token: &mut PkToken) {
    if token.stop { return; }
    if token.output_ptr >= token.output_length {
        unsafe { log_error("COMP2 Out of buffer space.\0".as_ptr() as *const c_char, ptr::null(), 0); }
        token.stop = true;
        return;
    }
    let len = buffer.len();
    if token.output_length - token.output_ptr >= len {
        unsafe {
            ptr::copy_nonoverlapping(buffer.as_ptr(), token.output_data.add(token.output_ptr), len);
        }
        token.output_ptr += len;
    } else {
        unsafe { log_error("COMP1 Corrupt.\0".as_ptr() as *const c_char, ptr::null(), 0); }
        token.stop = true;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn zip_compress(
    input_buffer: *const c_void,
    input_length: c_int,
    output_buffer: *mut c_void,
    output_length: *mut c_int,
) -> c_int {
    let mut token = PkToken {
        stop: false,
        input_data: input_buffer as *const u8,
        input_ptr: 0,
        input_length: input_length as usize,
        output_data: output_buffer as *mut u8,
        output_ptr: 0,
        output_length: unsafe { *output_length as usize },
    };

    let mut buf = PkCompBuffer {
        dictionary_size: 4096,
        window_size: 6,
        copy_offset_extra_mask: 0x3f,
        current_output_bits_used: 0,
        input_data: Box::new([0; 8708 + 516]),
        output_data: Box::new([0; 2050]),
        output_ptr: 0,
        analyze_offset_table: Box::new([0; 2304]),
        analyze_index: Box::new([0; 8708]),
        long_matcher: Box::new([0; 518]),
        codeword_values: Box::new([0; 774]),
        codeword_bits: Box::new([0; 774]),
    };

    for i in 0..256 {
        buf.codeword_bits[i] = 9;
        buf.codeword_values[i] = (i << 1) as u16;
    }
    let mut code_idx = 256;
    for copy in 0..16 {
        let base_bits = PK_COPY_LENGTH_BASE_BITS[copy];
        let extra_bits = PK_COPY_LENGTH_EXTRA_BITS[copy];
        let base_code = PK_COPY_LENGTH_BASE_CODE[copy];
        let max = 1 << extra_bits;
        for i in 0..max {
            buf.codeword_bits[code_idx] = 1 + base_bits + extra_bits;
            buf.codeword_values[code_idx] = 1 | ((base_code as u16) << 1) | (i << (base_bits + 1)) as u16;
            code_idx += 1;
        }
    }

    pk_implode_data(&mut buf, zip_input_func, zip_output_func, &mut token);
    if token.stop {
        return 0;
    }
    unsafe { *output_length = token.output_ptr as c_int; }
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn zip_decompress(
    input_buffer: *const c_void,
    input_length: c_int,
    output_buffer: *mut c_void,
    output_length: *mut c_int,
) -> c_int {
    let mut token = PkToken {
        stop: false,
        input_data: input_buffer as *const u8,
        input_ptr: 0,
        input_length: input_length as usize,
        output_data: output_buffer as *mut u8,
        output_ptr: 0,
        output_length: unsafe { *output_length as usize },
    };

    let mut buf = PkDecompBuffer {
        window_size: 0,
        dictionary_size: 0,
        current_input_byte: 0,
        current_input_bits_available: 0,
        input_buffer_ptr: 2048,
        input_buffer_end: 0,
        output_buffer_ptr: 4096,
        input_buffer: Box::new([0; 2048]),
        output_buffer: Box::new([0; 8708 + 516]),
        copy_offset_jump_table: Box::new([0; 256]),
        copy_length_jump_table: Box::new([0; 256]),
    };

    buf.input_buffer_end = zip_input_func(&mut *buf.input_buffer, &mut token);
    if buf.input_buffer_end <= 4 { return 0; }
    
    let has_literal = buf.input_buffer[0] != 0;
    buf.window_size = buf.input_buffer[1] as u32;
    buf.current_input_byte = buf.input_buffer[2] as u16;
    buf.current_input_bits_available = 0;
    buf.input_buffer_ptr = 3;

    if buf.window_size < 4 || buf.window_size > 6 { return 0; }
    buf.dictionary_size = 0xFFFF >> (16 - buf.window_size);
    if has_literal { return 0; }

    pk_explode_construct_jump_table(16, &PK_COPY_LENGTH_BASE_BITS, &PK_COPY_LENGTH_BASE_CODE, &mut buf.copy_length_jump_table);
    pk_explode_construct_jump_table(64, &PK_COPY_OFFSET_BITS, &PK_COPY_OFFSET_CODE, &mut buf.copy_offset_jump_table);

    let res = pk_explode_data(&mut buf, zip_input_func, zip_output_func, &mut token);
    if res != PK_EOF {
        return 0;
    }
    let remaining = buf.output_buffer_ptr - 4096;
    if remaining > 0 { zip_output_func(&buf.output_buffer[4096..buf.output_buffer_ptr], &mut token); }
    if token.stop { return 0; }
    unsafe { *output_length = token.output_ptr as c_int; }
    1
}
