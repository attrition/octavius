use std::ffi::c_int;
use crate::buffer::{Buffer, buffer_read_u32, buffer_write_u32};

const MAX_RANDOM: usize = 100;

struct RandomData {
    iv1: u32,
    iv2: u32,
    random1_7bit: i8,
    random1_15bit: i16,
    random2_7bit: i8,
    random2_15bit: i16,
    pool_index: usize,
    pool: [i32; MAX_RANDOM],
}

static mut DATA: RandomData = RandomData {
    iv1: 0,
    iv2: 0,
    random1_7bit: 0,
    random1_15bit: 0,
    random2_7bit: 0,
    random2_15bit: 0,
    pool_index: 0,
    pool: [0; MAX_RANDOM],
};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn random_init() {
    unsafe {
        DATA.iv1 = 0x54657687;
        DATA.iv2 = 0x72641663;
        DATA.random1_7bit = 0;
        DATA.random1_15bit = 0;
        DATA.random2_7bit = 0;
        DATA.random2_15bit = 0;
        DATA.pool_index = 0;
        for i in 0..MAX_RANDOM {
            DATA.pool[i] = 0;
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn random_generate_next() {
    unsafe {
        DATA.pool[DATA.pool_index] = DATA.random1_7bit as i32;
        DATA.pool_index += 1;
        if DATA.pool_index >= MAX_RANDOM {
            DATA.pool_index = 0;
        }

        for _ in 0..31 {
            let r1 = (((DATA.iv1 & 0x10) >> 4) ^ DATA.iv1) & 1;
            let r2 = (((DATA.iv2 & 0x10) >> 4) ^ DATA.iv2) & 1;
            DATA.iv1 >>= 1;
            DATA.iv2 >>= 1;
            if r1 != 0 {
                DATA.iv1 |= 0x40000000;
            }
            if r2 != 0 {
                DATA.iv2 |= 0x40000000;
            }
        }
        DATA.random1_7bit = (DATA.iv1 & 0x7f) as i8;
        DATA.random1_15bit = (DATA.iv1 & 0x7fff) as i16;
        DATA.random2_7bit = (DATA.iv2 & 0x7f) as i8;
        DATA.random2_15bit = (DATA.iv2 & 0x7fff) as i16;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn random_generate_pool() {
    unsafe {
        DATA.pool_index = 0;
        for _ in 0..MAX_RANDOM {
            random_generate_next();
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn random_byte() -> i8 {
    unsafe { DATA.random1_7bit }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn random_byte_alt() -> i8 {
    unsafe { DATA.random2_7bit }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn random_short() -> i16 {
    unsafe { DATA.random1_15bit }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn random_from_pool(index: c_int) -> i32 {
    unsafe {
        let idx = (DATA.pool_index + index as usize) % MAX_RANDOM;
        DATA.pool[idx]
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn random_load_state(buf: *mut Buffer) {
    unsafe {
        DATA.iv1 = buffer_read_u32(buf);
        DATA.iv2 = buffer_read_u32(buf);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn random_save_state(buf: *mut Buffer) {
    unsafe {
        buffer_write_u32(buf, DATA.iv1);
        buffer_write_u32(buf, DATA.iv2);
    }
}
