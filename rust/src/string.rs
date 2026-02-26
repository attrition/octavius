use std::ffi::{c_char, c_int};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn string_equals(a: *const u8, b: *const u8) -> c_int {
    if a.is_null() || b.is_null() {
        return (a == b) as c_int;
    }
    let mut curr_a = a;
    let mut curr_b = b;
    unsafe {
        while *curr_a != 0 && *curr_b != 0 && *curr_a == *curr_b {
            curr_a = curr_a.add(1);
            curr_b = curr_b.add(1);
        }
        (*curr_a == 0 && *curr_b == 0) as c_int
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn string_copy(src: *const u8, dst: *mut u8, maxlength: c_int) {
    if dst.is_null() || src.is_null() {
        return;
    }
    let mut length = 0;
    let mut curr_src = src;
    let mut curr_dst = dst;
    unsafe {
        while length < maxlength && *curr_src != 0 {
            *curr_dst = *curr_src;
            curr_src = curr_src.add(1);
            curr_dst = curr_dst.add(1);
            length += 1;
        }
        if length == maxlength {
            curr_dst = curr_dst.sub(1);
        }
        *curr_dst = 0;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn string_length(str: *const u8) -> c_int {
    if str.is_null() {
        return 0;
    }
    let mut length = 0;
    let mut curr = str;
    unsafe {
        while *curr != 0 {
            length += 1;
            curr = curr.add(1);
        }
    }
    length
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn string_from_ascii(str: *const c_char) -> *const u8 {
    if str.is_null() {
        return std::ptr::null();
    }
    let mut curr = str;
    unsafe {
        while *curr != 0 {
            if (*curr as u8) & 0x80 != 0 {
                return std::ptr::null();
            }
            curr = curr.add(1);
        }
    }
    str as *const u8
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn string_to_int(str: *const u8) -> c_int {
    if str.is_null() {
        return 0;
    }
    const MULTIPLIERS: [c_int; 8] = [1, 10, 100, 1000, 10000, 100000, 1000000, 10000000];
    let mut ptr = str;
    let mut negative = false;
    let mut num_chars = 0;
    unsafe {
        if *ptr == b'-' {
            negative = true;
            ptr = ptr.add(1);
        }
        let mut check_ptr = ptr;
        while *check_ptr >= b'0' && *check_ptr <= b'9' {
            num_chars += 1;
            check_ptr = check_ptr.add(1);
        }

        if num_chars > 8 {
            return 0;
        }

        let mut curr_ptr = ptr;
        let mut result = 0;
        let mut i = num_chars;
        while i > 0 {
            i -= 1;
            result += MULTIPLIERS[i] * ((*curr_ptr - b'0') as c_int);
            curr_ptr = curr_ptr.add(1);
        }
        if negative {
            result = -result;
        }
        result
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn string_from_int(dst: *mut u8, value: c_int, force_plus_sign: c_int) -> c_int {
    if dst.is_null() {
        return 0;
    }
    let mut total_chars = 0;
    let mut val = value;
    let mut curr_dst = dst;
    unsafe {
        if val >= 0 {
            if force_plus_sign != 0 {
                *curr_dst = b'+';
                curr_dst = curr_dst.add(1);
                total_chars = 1;
            }
        } else {
            *curr_dst = b'-';
            curr_dst = curr_dst.add(1);
            val = -val;
            total_chars = 1;
        }

        let num_digits = if val < 10 {
            1
        } else if val < 100 {
            2
        } else if val < 1000 {
            3
        } else if val < 10000 {
            4
        } else if val < 100000 {
            5
        } else if val < 1000000 {
            6
        } else if val < 10000000 {
            7
        } else if val < 100000000 {
            8
        } else if val < 1000000000 {
            9
        } else {
            0
        };

        total_chars += num_digits;
        *curr_dst.add(num_digits as usize) = 0;
        let mut i = num_digits;
        let mut v = val;
        while i > 0 {
            i -= 1;
            *curr_dst.add(i as usize) = (v % 10) as u8 + b'0';
            v /= 10;
        }

        total_chars
    }
}
