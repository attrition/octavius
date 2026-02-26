use std::ffi::{c_char, c_int, c_void};
use std::ptr;

unsafe extern "C" {
    fn platform_file_manager_open_file(filename: *const c_char, mode: *const c_char) -> *mut c_void;
    fn platform_file_manager_close_file(stream: *mut c_void) -> c_int;
    fn platform_file_manager_compare_filename(a: *const c_char, b: *const c_char) -> c_int;
    fn platform_file_manager_remove_file(filename: *const c_char) -> c_int;
    fn dir_get_file(filepath: *const c_char, localizable: c_int) -> *const c_char;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_open(filename: *const c_char, mode: *const c_char) -> *mut c_void {
    unsafe { platform_file_manager_open_file(filename, mode) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_close(stream: *mut c_void) -> c_int {
    unsafe { platform_file_manager_close_file(stream) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_has_extension(filename: *const c_char, extension: *const c_char) -> c_int {
    unsafe {
        if extension.is_null() || *extension == 0 {
            return 1;
        }
        let mut curr = filename;
        let mut c = *curr;
        while c != b'.' as c_char && c != 0 {
            curr = curr.add(1);
            c = *curr;
        }
        if c == 0 {
            // filename has no dot
        } else {
            curr = curr.add(1); // point to after the dot
        }
        (platform_file_manager_compare_filename(curr, extension) == 0) as c_int
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_change_extension(filename: *mut c_char, new_extension: *const c_char) {
    unsafe {
        let mut curr = filename;
        let mut c = *curr;
        while c != b'.' as c_char && c != 0 {
            curr = curr.add(1);
            c = *curr;
        }
        if c == b'.' as c_char {
            curr = curr.add(1);
            *curr.add(0) = *new_extension.add(0);
            *curr.add(1) = *new_extension.add(1);
            *curr.add(2) = *new_extension.add(2);
            *curr.add(3) = 0;
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_append_extension(filename: *mut c_char, extension: *const c_char) {
    unsafe {
        let mut curr = filename;
        while *curr != 0 {
            curr = curr.add(1);
        }
        *curr.add(0) = b'.' as c_char;
        *curr.add(1) = *extension.add(0);
        *curr.add(2) = *extension.add(1);
        *curr.add(3) = *extension.add(2);
        *curr.add(4) = 0;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_remove_extension(filename: *mut u8) {
    unsafe {
        let mut curr = filename;
        let mut c = *curr;
        while c != b'.' && c != 0 {
            curr = curr.add(1);
            c = *curr;
        }
        if c == b'.' {
            *curr = 0;
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_exists(filename: *const c_char, localizable: c_int) -> c_int {
    unsafe { (!dir_get_file(filename, localizable).is_null()) as c_int }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_remove(filename: *const c_char) -> c_int {
    unsafe { platform_file_manager_remove_file(filename) }
}
