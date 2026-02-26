use std::ffi::c_int;
use std::ptr::{addr_of, addr_of_mut};
use crate::color::ColorT;

struct ScreenData {
    width: i32,
    height: i32,
    dialog_offset_x: i32,
    dialog_offset_y: i32,
}

static mut DATA: ScreenData = ScreenData {
    width: 0,
    height: 0,
    dialog_offset_x: 0,
    dialog_offset_y: 0,
};

unsafe extern "C" {
    fn graphics_init_canvas(width: c_int, height: c_int);
    fn city_view_set_viewport(width: c_int, height: c_int);
    fn city_warning_clear_all();
    fn window_invalidate();
    fn graphics_get_pixel(x: c_int, y: c_int) -> *mut ColorT;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn screen_set_resolution(width: c_int, height: c_int) {
    unsafe {
        let d = addr_of_mut!(DATA);
        (*d).width = width;
        (*d).height = height;
        (*d).dialog_offset_x = (width - 640) / 2;
        (*d).dialog_offset_y = (height - 480) / 2;
        
        graphics_init_canvas(width, height);
        city_view_set_viewport(width, height);
        city_warning_clear_all();
        window_invalidate();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn screen_pixel(x: c_int, y: c_int) -> *mut ColorT {
    unsafe { graphics_get_pixel(x, y) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn screen_width() -> c_int {
    unsafe { (*addr_of!(DATA)).width }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn screen_height() -> c_int {
    unsafe { (*addr_of!(DATA)).height }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn screen_dialog_offset_x() -> c_int {
    unsafe { (*addr_of!(DATA)).dialog_offset_x }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn screen_dialog_offset_y() -> c_int {
    unsafe { (*addr_of!(DATA)).dialog_offset_y }
}
