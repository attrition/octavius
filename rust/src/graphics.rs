use std::ffi::{c_int, c_void};
use std::ptr::{self, addr_of, addr_of_mut};
use crate::color::{ColorT, COLOR_INSET_DARK, COLOR_INSET_LIGHT};

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ClipCode {
    None = 0,
    Left = 1,
    Right = 2,
    Top = 3,
    Bottom = 4,
    Both = 5,
    Invisible = 6,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ClipInfo {
    pub clip_x: ClipCode,
    pub clip_y: ClipCode,
    pub clipped_pixels_left: c_int,
    pub clipped_pixels_right: c_int,
    pub clipped_pixels_top: c_int,
    pub clipped_pixels_bottom: c_int,
    pub visible_pixels_x: c_int,
    pub visible_pixels_y: c_int,
    pub is_visible: c_int,
}

struct Canvas {
    pixels: *mut ColorT,
    width: i32,
    height: i32,
}

struct ClipRectangle {
    x_start: i32,
    x_end: i32,
    y_start: i32,
    y_end: i32,
}

struct Translation {
    x: i32,
    y: i32,
}

static mut CANVAS: Canvas = Canvas {
    pixels: ptr::null_mut(),
    width: 0,
    height: 0,
};

static mut CLIP_RECTANGLE: ClipRectangle = ClipRectangle {
    x_start: 0,
    x_end: 800,
    y_start: 0,
    y_end: 600,
};

static mut TRANSLATION: Translation = Translation { x: 0, y: 0 };

static mut CLIP: ClipInfo = ClipInfo {
    clip_x: ClipCode::None,
    clip_y: ClipCode::None,
    clipped_pixels_left: 0,
    clipped_pixels_right: 0,
    clipped_pixels_top: 0,
    clipped_pixels_bottom: 0,
    visible_pixels_x: 0,
    visible_pixels_y: 0,
    is_visible: 0,
};

unsafe extern "C" {
    fn system_create_framebuffer(width: c_int, height: c_int) -> *mut ColorT;
    fn screen_dialog_offset_x() -> c_int;
    fn screen_dialog_offset_y() -> c_int;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn graphics_init_canvas(width: c_int, height: c_int) {
    unsafe {
        (*addr_of_mut!(CANVAS.pixels)) = system_create_framebuffer(width, height);
        ptr::write_bytes(*addr_of!(CANVAS.pixels), 0, (width * height) as usize);
        (*addr_of_mut!(CANVAS.width)) = width;
        (*addr_of_mut!(CANVAS.height)) = height;
        graphics_set_clip_rectangle(0, 0, width, height);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn graphics_canvas() -> *const c_void {
    unsafe { (*addr_of!(CANVAS.pixels)) as *const c_void }
}

unsafe fn translate_clip(dx: i32, dy: i32) {
    unsafe {
        (*addr_of_mut!(CLIP_RECTANGLE.x_start)) -= dx;
        (*addr_of_mut!(CLIP_RECTANGLE.x_end)) -= dx;
        (*addr_of_mut!(CLIP_RECTANGLE.y_start)) -= dy;
        (*addr_of_mut!(CLIP_RECTANGLE.y_end)) -= dy;
    }
}

unsafe fn set_translation(x: i32, y: i32) {
    unsafe {
        let dx = x - (*addr_of!(TRANSLATION.x));
        let dy = y - (*addr_of!(TRANSLATION.y));
        (*addr_of_mut!(TRANSLATION.x)) = x;
        (*addr_of_mut!(TRANSLATION.y)) = y;
        translate_clip(dx, dy);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn graphics_in_dialog() {
    unsafe {
        set_translation(screen_dialog_offset_x(), screen_dialog_offset_y());
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn graphics_reset_dialog() {
    unsafe {
        set_translation(0, 0);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn graphics_set_clip_rectangle(x: c_int, y: c_int, width: c_int, height: c_int) {
    unsafe {
        let cr = addr_of_mut!(CLIP_RECTANGLE);
        (*cr).x_start = x;
        (*cr).x_end = x + width;
        (*cr).y_start = y;
        (*cr).y_end = y + height;
        
        let tx = *addr_of!(TRANSLATION.x);
        let ty = *addr_of!(TRANSLATION.y);
        let cw = *addr_of!(CANVAS.width);
        let ch = *addr_of!(CANVAS.height);
        
        if tx + (*cr).x_start < 0 { (*cr).x_start = -tx; }
        if ty + (*cr).y_start < 0 { (*cr).y_start = -ty; }
        if tx + (*cr).x_end > cw { (*cr).x_end = cw - tx; }
        if ty + (*cr).y_end > ch { (*cr).y_end = ch - ty; }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn graphics_reset_clip_rectangle() {
    unsafe {
        let cr = addr_of_mut!(CLIP_RECTANGLE);
        let cw = *addr_of!(CANVAS.width);
        let ch = *addr_of!(CANVAS.height);
        (*cr).x_start = 0;
        (*cr).x_end = cw;
        (*cr).y_start = 0;
        (*cr).y_end = ch;
        translate_clip(*addr_of!(TRANSLATION.x), *addr_of!(TRANSLATION.y));
    }
}

unsafe fn set_clip_x(x_offset: i32, width: i32) {
    unsafe {
        let c = addr_of_mut!(CLIP);
        let cr = addr_of!(CLIP_RECTANGLE);
        (*c).clipped_pixels_left = 0;
        (*c).clipped_pixels_right = 0;
        if width <= 0 || x_offset + width <= (*cr).x_start || x_offset >= (*cr).x_end {
            (*c).clip_x = ClipCode::Invisible;
            (*c).visible_pixels_x = 0;
            return;
        }
        if x_offset < (*cr).x_start {
            (*c).clipped_pixels_left = (*cr).x_start - x_offset;
            if x_offset + width <= (*cr).x_end {
                (*c).clip_x = ClipCode::Left;
            } else {
                (*c).clip_x = ClipCode::Both;
                (*c).clipped_pixels_right = x_offset + width - (*cr).x_end;
            }
        } else if x_offset + width > (*cr).x_end {
            (*c).clip_x = ClipCode::Right;
            (*c).clipped_pixels_right = x_offset + width - (*cr).x_end;
        } else {
            (*c).clip_x = ClipCode::None;
        }
        (*c).visible_pixels_x = width - (*c).clipped_pixels_left - (*c).clipped_pixels_right;
    }
}

unsafe fn set_clip_y(y_offset: i32, height: i32) {
    unsafe {
        let c = addr_of_mut!(CLIP);
        let cr = addr_of!(CLIP_RECTANGLE);
        (*c).clipped_pixels_top = 0;
        (*c).clipped_pixels_bottom = 0;
        if height <= 0 || y_offset + height <= (*cr).y_start || y_offset >= (*cr).y_end {
            (*c).clip_y = ClipCode::Invisible;
        } else if y_offset < (*cr).y_start {
            (*c).clipped_pixels_top = (*cr).y_start - y_offset;
            if y_offset + height <= (*cr).y_end {
                (*c).clip_y = ClipCode::Top;
            } else {
                (*c).clip_y = ClipCode::Both;
                (*c).clipped_pixels_bottom = y_offset + height - (*cr).y_end;
            }
        } else if y_offset + height > (*cr).y_end {
            (*c).clip_y = ClipCode::Bottom;
            (*c).clipped_pixels_bottom = y_offset + height - (*cr).y_end;
        } else {
            (*c).clip_y = ClipCode::None;
        }
        (*c).visible_pixels_y = height - (*c).clipped_pixels_top - (*c).clipped_pixels_bottom;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn graphics_get_clip_info(x: c_int, y: c_int, width: c_int, height: c_int) -> *const ClipInfo {
    unsafe {
        set_clip_x(x, width);
        set_clip_y(y, height);
        let c = addr_of_mut!(CLIP);
        if (*c).clip_x == ClipCode::Invisible || (*c).clip_y == ClipCode::Invisible {
            (*c).is_visible = 0;
        } else {
            (*c).is_visible = 1;
        }
        c
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn graphics_get_pixel(x: c_int, y: c_int) -> *mut ColorT {
    unsafe {
        let canvas = addr_of!(CANVAS);
        let trans = addr_of!(TRANSLATION);
        (*canvas).pixels.add((( (*trans).y + y ) * (*canvas).width + ( (*trans).x + x )) as usize)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn graphics_save_to_buffer(x: c_int, y: c_int, width: c_int, height: c_int, buffer: *mut ColorT) {
    unsafe {
        let clip_ptr = graphics_get_clip_info(x, y, width, height);
        if (*clip_ptr).is_visible == 0 { return; }
        
        let min_x = x + (*clip_ptr).clipped_pixels_left;
        let min_dy = (*clip_ptr).clipped_pixels_top;
        let max_dy = height - (*clip_ptr).clipped_pixels_bottom;
        let visible_x = (*clip_ptr).visible_pixels_x;
        
        for dy in min_dy..max_dy {
            ptr::copy_nonoverlapping(
                graphics_get_pixel(min_x, y + dy),
                buffer.add((dy * width) as usize),
                visible_x as usize
            );
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn graphics_draw_from_buffer(x: c_int, y: c_int, width: c_int, height: c_int, buffer: *const ColorT) {
    unsafe {
        let clip_ptr = graphics_get_clip_info(x, y, width, height);
        if (*clip_ptr).is_visible == 0 { return; }
        
        let min_x = x + (*clip_ptr).clipped_pixels_left;
        let min_dy = (*clip_ptr).clipped_pixels_top;
        let max_dy = height - (*clip_ptr).clipped_pixels_bottom;
        let visible_x = (*clip_ptr).visible_pixels_x;
        
        for dy in min_dy..max_dy {
            ptr::copy_nonoverlapping(
                buffer.add((dy * width) as usize),
                graphics_get_pixel(min_x, y + dy),
                visible_x as usize
            );
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn graphics_clear_screen() {
    unsafe {
        let c = addr_of!(CANVAS);
        ptr::write_bytes((*c).pixels, 0, ((*c).width * (*c).height) as usize);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn graphics_draw_vertical_line(x: c_int, y1: c_int, y2: c_int, color: ColorT) {
    unsafe {
        let cr = addr_of!(CLIP_RECTANGLE);
        if x < (*cr).x_start || x >= (*cr).x_end { return; }
        
        let mut y_min = if y1 < y2 { y1 } else { y2 };
        let mut y_max = if y1 < y2 { y2 } else { y1 };
        
        if y_min < (*cr).y_start { y_min = (*cr).y_start; }
        if y_max >= (*cr).y_end { y_max = (*cr).y_end - 1; }
        
        if y_min > y_max { return; }
        
        let mut pixel = graphics_get_pixel(x, y_min);
        let cw = (*addr_of!(CANVAS)).width as usize;
        for _ in 0..=(y_max - y_min) {
            *pixel = color;
            pixel = pixel.add(cw);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn graphics_draw_horizontal_line(x1: c_int, x2: c_int, y: c_int, color: ColorT) {
    unsafe {
        let cr = addr_of!(CLIP_RECTANGLE);
        if y < (*cr).y_start || y >= (*cr).y_end { return; }
        
        let mut x_min = if x1 < x2 { x1 } else { x2 };
        let mut x_max = if x1 < x2 { x2 } else { x1 };
        
        if x_min < (*cr).x_start { x_min = (*cr).x_start; }
        if x_max >= (*cr).x_end { x_max = (*cr).x_end - 1; }
        
        if x_min > x_max { return; }
        
        let pixel = graphics_get_pixel(x_min, y);
        for i in 0..=(x_max - x_min) {
            *pixel.add(i as usize) = color;
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn graphics_draw_rect(x: c_int, y: c_int, width: c_int, height: c_int, color: ColorT) {
    unsafe {
        graphics_draw_horizontal_line(x, x + width - 1, y, color);
        graphics_draw_horizontal_line(x, x + width - 1, y + height - 1, color);
        graphics_draw_vertical_line(x, y, y + height - 1, color);
        graphics_draw_vertical_line(x + width - 1, y, y + height - 1, color);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn graphics_draw_inset_rect(x: c_int, y: c_int, width: c_int, height: c_int) {
    unsafe {
        graphics_draw_horizontal_line(x, x + width - 1, y, COLOR_INSET_DARK);
        graphics_draw_vertical_line(x + width - 1, y, y + height - 1, COLOR_INSET_LIGHT);
        graphics_draw_horizontal_line(x, x + width - 1, y + height - 1, COLOR_INSET_LIGHT);
        graphics_draw_vertical_line(x, y, y + height - 1, COLOR_INSET_DARK);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn graphics_fill_rect(x: c_int, y: c_int, width: c_int, height: c_int, color: ColorT) {
    unsafe {
        for yy in y..(y + height) {
            graphics_draw_horizontal_line(x, x + width - 1, yy, color);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn graphics_shade_rect(x: c_int, y: c_int, width: c_int, height: c_int, darkness: c_int) {
    unsafe {
        let clip_ptr = graphics_get_clip_info(x, y, width, height);
        if (*clip_ptr).is_visible == 0 { return; }
        
        let start_y = y + (*clip_ptr).clipped_pixels_top;
        let end_y = y + height - (*clip_ptr).clipped_pixels_bottom;
        let start_x = x + (*clip_ptr).clipped_pixels_left;
        let end_x = x + width - (*clip_ptr).clipped_pixels_right;
        
        for yy in start_y..end_y {
            for xx in start_x..end_x {
                let pixel = graphics_get_pixel(xx, yy);
                let p = *pixel;
                let r = (p & 0xff0000) >> 16;
                let g = (p & 0xff00) >> 8;
                let b = p & 0xff;
                let grey = ((r + g + b) / 3) >> darkness;
                *pixel = (grey << 16) | (grey << 8) | grey;
            }
        }
    }
}
