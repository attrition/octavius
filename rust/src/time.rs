use std::ffi::c_uint;

pub type TimeMillis = c_uint;

static mut CURRENT_TIME: TimeMillis = 0;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn time_get_millis() -> TimeMillis {
    unsafe { CURRENT_TIME }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn time_set_millis(millis: TimeMillis) {
    unsafe {
        CURRENT_TIME = millis;
    }
}
