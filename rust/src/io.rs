use std::ffi::{c_char, c_int, c_long, c_void};

// Standard C library functions used in io.c
unsafe extern "C" {
    fn fseek(stream: *mut c_void, offset: c_long, whence: c_int) -> c_int;
    fn ftell(stream: *mut c_void) -> c_long;
    fn fread(ptr: *mut c_void, size: usize, nmemb: usize, stream: *mut c_void) -> usize;
    fn fwrite(ptr: *const c_void, size: usize, nmemb: usize, stream: *mut c_void) -> usize;
}

const SEEK_SET: c_int = 0;
const SEEK_END: c_int = 2;

// octavius core functions
unsafe extern "C" {
    fn dir_get_file(filepath: *const c_char, localizable: c_int) -> *const c_char;
    fn file_open(filename: *const c_char, mode: *const c_char) -> *mut c_void;
    fn file_close(stream: *mut c_void) -> c_int;
}

const NOT_LOCALIZED: c_int = 0;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn io_read_file_into_buffer(
    filepath: *const c_char,
    localizable: c_int,
    buffer: *mut c_void,
    max_size: c_int,
) -> c_int {
    let cased_file = unsafe { dir_get_file(filepath, localizable) };
    if cased_file.is_null() {
        return 0;
    }
    let fp = unsafe { file_open(cased_file, "rb\0".as_ptr() as *const c_char) };
    if fp.is_null() {
        return 0;
    }
    unsafe {
        fseek(fp, 0, SEEK_END);
        let mut size = ftell(fp);
        if size > max_size as c_long {
            size = max_size as c_long;
        }
        fseek(fp, 0, SEEK_SET);
        let bytes_read = fread(buffer, 1, size as usize, fp);
        file_close(fp);
        bytes_read as c_int
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn io_read_file_part_into_buffer(
    filepath: *const c_char,
    localizable: c_int,
    buffer: *mut c_void,
    size: c_int,
    offset_in_file: c_int,
) -> c_int {
    let cased_file = unsafe { dir_get_file(filepath, localizable) };
    if cased_file.is_null() {
        return 0;
    }
    let mut bytes_read = 0;
    let fp = unsafe { file_open(cased_file, "rb\0".as_ptr() as *const c_char) };
    if !fp.is_null() {
        unsafe {
            let seek_result = fseek(fp, offset_in_file as c_long, SEEK_SET);
            if seek_result == 0 {
                bytes_read = fread(buffer, 1, size as usize, fp);
            }
            file_close(fp);
        }
    }
    bytes_read as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn io_write_buffer_to_file(
    filepath: *const c_char,
    buffer: *const c_void,
    size: c_int,
) -> c_int {
    let mut cased_file = unsafe { dir_get_file(filepath, NOT_LOCALIZED) };
    if cased_file.is_null() {
        cased_file = filepath;
    }
    let fp = unsafe { file_open(cased_file, "wb\0".as_ptr() as *const c_char) };
    if fp.is_null() {
        return 0;
    }
    unsafe {
        let bytes_written = fwrite(buffer, 1, size as usize, fp);
        file_close(fp);
        bytes_written as c_int
    }
}
