use std::ffi::{c_int, c_void};
use std::ptr;

#[repr(C)]
pub struct Buffer {
    pub data: *mut u8,
    pub size: c_int,
    pub index: c_int,
    pub overflow: c_int,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn buffer_init(buf: *mut Buffer, data: *mut c_void, size: c_int) {
    if buf.is_null() { return; }
    let b = unsafe { &mut *buf };
    b.data = data as *mut u8;
    b.size = size;
    b.index = 0;
    b.overflow = 0;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn buffer_reset(buf: *mut Buffer) {
    if buf.is_null() { return; }
    let b = unsafe { &mut *buf };
    b.index = 0;
    b.overflow = 0;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn buffer_set(buf: *mut Buffer, offset: c_int) {
    if buf.is_null() { return; }
    let b = unsafe { &mut *buf };
    b.index = offset;
}

#[inline]
unsafe fn check_size(buf: *mut Buffer, size: c_int) -> bool {
    let b = unsafe { &mut *buf };
    if b.index + size > b.size {
        b.overflow = 1;
        false
    } else {
        true
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn buffer_write_u8(buf: *mut Buffer, value: u8) {
    if unsafe { check_size(buf, 1) } {
        let b = unsafe { &mut *buf };
        unsafe {
            *b.data.add(b.index as usize) = value;
        }
        b.index += 1;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn buffer_write_u16(buf: *mut Buffer, value: u16) {
    if unsafe { check_size(buf, 2) } {
        let b = unsafe { &mut *buf };
        unsafe {
            *b.data.add(b.index as usize) = (value & 0xff) as u8;
            *b.data.add(b.index as usize + 1) = ((value >> 8) & 0xff) as u8;
        }
        b.index += 2;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn buffer_write_u32(buf: *mut Buffer, value: u32) {
    if unsafe { check_size(buf, 4) } {
        let b = unsafe { &mut *buf };
        unsafe {
            *b.data.add(b.index as usize) = (value & 0xff) as u8;
            *b.data.add(b.index as usize + 1) = ((value >> 8) & 0xff) as u8;
            *b.data.add(b.index as usize + 2) = ((value >> 16) & 0xff) as u8;
            *b.data.add(b.index as usize + 3) = ((value >> 24) & 0xff) as u8;
        }
        b.index += 4;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn buffer_write_i8(buf: *mut Buffer, value: i8) {
    unsafe {
        buffer_write_u8(buf, value as u8);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn buffer_write_i16(buf: *mut Buffer, value: i16) {
    unsafe {
        buffer_write_u16(buf, value as u16);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn buffer_write_i32(buf: *mut Buffer, value: i32) {
    unsafe {
        buffer_write_u32(buf, value as u32);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn buffer_write_raw(buf: *mut Buffer, value: *const c_void, size: c_int) {
    if unsafe { check_size(buf, size) } {
        let b = unsafe { &mut *buf };
        unsafe {
            ptr::copy_nonoverlapping(value as *const u8, b.data.add(b.index as usize), size as usize);
        }
        b.index += size;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn buffer_read_u8(buf: *mut Buffer) -> u8 {
    if unsafe { check_size(buf, 1) } {
        let b = unsafe { &mut *buf };
        let val = unsafe { *b.data.add(b.index as usize) };
        b.index += 1;
        val
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn buffer_read_u16(buf: *mut Buffer) -> u16 {
    if unsafe { check_size(buf, 2) } {
        let b = unsafe { &mut *buf };
        let b0 = unsafe { *b.data.add(b.index as usize) } as u16;
        let b1 = unsafe { *b.data.add(b.index as usize + 1) } as u16;
        b.index += 2;
        b0 | (b1 << 8)
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn buffer_read_u32(buf: *mut Buffer) -> u32 {
    if unsafe { check_size(buf, 4) } {
        let b = unsafe { &mut *buf };
        let b0 = unsafe { *b.data.add(b.index as usize) } as u32;
        let b1 = unsafe { *b.data.add(b.index as usize + 1) } as u32;
        let b2 = unsafe { *b.data.add(b.index as usize + 2) } as u32;
        let b3 = unsafe { *b.data.add(b.index as usize + 3) } as u32;
        b.index += 4;
        b0 | (b1 << 8) | (b2 << 16) | (b3 << 24)
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn buffer_read_i8(buf: *mut Buffer) -> i8 {
    unsafe { buffer_read_u8(buf) as i8 }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn buffer_read_i16(buf: *mut Buffer) -> i16 {
    unsafe { buffer_read_u16(buf) as i16 }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn buffer_read_i32(buf: *mut Buffer) -> i32 {
    unsafe { buffer_read_u32(buf) as i32 }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn buffer_read_raw(buf: *mut Buffer, value: *mut c_void, max_size: c_int) -> c_int {
    if buf.is_null() { return 0; }
    let b = unsafe { &mut *buf };
    let mut size = b.size - b.index;
    if size > max_size {
        size = max_size;
    }
    if size > 0 {
        unsafe {
            ptr::copy_nonoverlapping(b.data.add(b.index as usize), value as *mut u8, size as usize);
        }
        b.index += size;
        size
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn buffer_skip(buf: *mut Buffer, size: c_int) {
    if buf.is_null() { return; }
    let b = unsafe { &mut *buf };
    b.index += size;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn buffer_at_end(buf: *mut Buffer) -> c_int {
    if buf.is_null() { return 1; }
    let b = unsafe { &mut *buf };
    (b.index >= b.size) as c_int
}
