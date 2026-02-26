use std::ffi::c_int;
use std::ptr;
use crate::color::{ColorT, COLOR_BLACK, COLOR_WHITE};
use crate::graphics::{graphics_get_clip_info, graphics_get_pixel, graphics_clear_screen, ClipCode};
use crate::image::{Image, image_get, image_data, image_get_enemy, image_data_enemy, image_letter, image_data_letter};
use crate::screen::{screen_width, screen_height};

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FontT {
    NormalPlain = 0,
    NormalBlack = 1,
    NormalWhite = 2,
    NormalRed = 3,
    LargePlain = 4,
    LargeBlack = 5,
    LargeBrown = 6,
    SmallPlain = 7,
    NormalGreen = 8,
    NormalBROWN = 9,
    MaxItems = 10,
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum DrawType {
    Set = 0,
    And = 1,
    None = 2,
    Blend = 3,
    BlendAlpha = 4,
}

const FOOTPRINT_WIDTH: i32 = 58;
const FOOTPRINT_HEIGHT: i32 = 30;

const COLOR_SG2_TRANSPARENT: ColorT = 0xf700ff;
const IMAGE_TYPE_ISOMETRIC: i32 = 2;
const IMAGE_TYPE_WITH_TRANSPARENCY: i32 = 1;
const IMAGE_FONT_MULTIBYTE_OFFSET: i32 = 10000;

#[inline]
fn component(c: ColorT, shift: u32) -> ColorT {
    (c >> shift) & 0xff
}

#[inline]
fn mix_rb(src: ColorT, dst: ColorT, alpha: ColorT) -> ColorT {
    (((src & 0xff00ff) * alpha + (dst & 0xff00ff) * (256 - alpha)) >> 8) & 0xff00ff
}

#[inline]
fn mix_g(src: ColorT, dst: ColorT, alpha: ColorT) -> ColorT {
    (((src & 0x00ff00) * alpha + (dst & 0x00ff00) * (256 - alpha)) >> 8) & 0x00ff00
}

static FOOTPRINT_X_START_PER_HEIGHT: [i32; 30] = [
    28, 26, 24, 22, 20, 18, 16, 14, 12, 10, 8, 6, 4, 2, 0,
    0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28
];

static FOOTPRINT_OFFSET_PER_HEIGHT: [i32; 30] = [
    0, 2, 8, 18, 32, 50, 72, 98, 128, 162, 200, 242, 288, 338, 392, 450,
    508, 562, 612, 658, 700, 738, 772, 802, 828, 850, 868, 882, 892, 898
];

unsafe fn draw_uncompressed(
    img: *const Image, data: *const ColorT, x_offset: c_int, y_offset: c_int, color: ColorT, dtype: DrawType
) {
    unsafe {
        let clip = graphics_get_clip_info(x_offset, y_offset, (*img).width, (*img).height);
        if (*clip).is_visible == 0 { return; }
        
        let mut src = data.add(((*img).width * (*clip).clipped_pixels_top) as usize);
        let x_max = (*img).width - (*clip).clipped_pixels_right;
        
        for y in (*clip).clipped_pixels_top .. ((*img).height - (*clip).clipped_pixels_bottom) {
            src = src.add((*clip).clipped_pixels_left as usize);
            let mut dst = graphics_get_pixel(x_offset + (*clip).clipped_pixels_left, y_offset + y);
            
            match dtype {
                DrawType::None => {
                    if (*img).draw.draw_type == IMAGE_TYPE_WITH_TRANSPARENCY || (*img).draw.is_external != 0 {
                        for _ in (*clip).clipped_pixels_left .. x_max {
                            if *src != COLOR_SG2_TRANSPARENT { *dst = *src; }
                            src = src.add(1);
                            dst = dst.add(1);
                        }
                    } else {
                        let num = (x_max - (*clip).clipped_pixels_left) as usize;
                        ptr::copy_nonoverlapping(src, dst, num);
                        src = src.add(num);
                    }
                }
                DrawType::Set => {
                    for _ in (*clip).clipped_pixels_left .. x_max {
                        if *src != COLOR_SG2_TRANSPARENT { *dst = color; }
                        src = src.add(1);
                        dst = dst.add(1);
                    }
                }
                DrawType::And => {
                    for _ in (*clip).clipped_pixels_left .. x_max {
                        if *src != COLOR_SG2_TRANSPARENT { *dst = *src & color; }
                        src = src.add(1);
                        dst = dst.add(1);
                    }
                }
                DrawType::Blend => {
                    for _ in (*clip).clipped_pixels_left .. x_max {
                        if *src != COLOR_SG2_TRANSPARENT { *dst &= color; }
                        src = src.add(1);
                        dst = dst.add(1);
                    }
                }
                DrawType::BlendAlpha => {
                    for _ in (*clip).clipped_pixels_left .. x_max {
                        if *src != COLOR_SG2_TRANSPARENT {
                            let alpha = component(*src, 24);
                            if alpha == 255 {
                                *dst = color;
                            } else {
                                *dst = mix_rb(color, *dst, alpha) | mix_g(color, *dst, alpha);
                            }
                        }
                        src = src.add(1);
                        dst = dst.add(1);
                    }
                }
            }
            src = src.add((*clip).clipped_pixels_right as usize);
        }
    }
}

unsafe fn draw_compressed(img: *const Image, mut data: *const ColorT, x_offset: c_int, y_offset: c_int, height: c_int) {
    unsafe {
        let clip = graphics_get_clip_info(x_offset, y_offset, (*img).width, height);
        if (*clip).is_visible == 0 { return; }
        let unclipped = (*clip).clip_x == ClipCode::None;
        
        for y in 0 .. (height - (*clip).clipped_pixels_bottom) {
            let mut x = 0;
            while x < (*img).width {
                let mut b = *data;
                data = data.add(1);
                if b == 255 {
                    x += *data as i32;
                    data = data.add(1);
                } else if y < (*clip).clipped_pixels_top {
                    data = data.add(b as usize);
                    x += b as i32;
                } else {
                    let pixels = data;
                    data = data.add(b as usize);
                    let mut dst = graphics_get_pixel(x_offset + x, y_offset + y);
                    if unclipped {
                        x += b as i32;
                        ptr::copy_nonoverlapping(pixels, dst, b as usize);
                    } else {
                        while b > 0 {
                            if x >= (*clip).clipped_pixels_left && x < ((*img).width - (*clip).clipped_pixels_right) {
                                *dst = *pixels;
                            }
                            dst = dst.add(1);
                            x += 1;
                            b -= 1;
                        }
                    }
                }
            }
        }
    }
}

unsafe fn draw_compressed_set(img: *const Image, mut data: *const ColorT, x_offset: c_int, y_offset: c_int, height: c_int, color: ColorT) {
    unsafe {
        let clip = graphics_get_clip_info(x_offset, y_offset, (*img).width, height);
        if (*clip).is_visible == 0 { return; }
        let unclipped = (*clip).clip_x == ClipCode::None;
        
        for y in 0 .. (height - (*clip).clipped_pixels_bottom) {
            let mut x = 0;
            while x < (*img).width {
                let mut b = *data;
                data = data.add(1);
                if b == 255 {
                    x += *data as i32;
                    data = data.add(1);
                } else if y < (*clip).clipped_pixels_top {
                    data = data.add(b as usize);
                    x += b as i32;
                } else {
                    data = data.add(b as usize);
                    let mut dst = graphics_get_pixel(x_offset + x, y_offset + y);
                    if unclipped {
                        x += b as i32;
                        while b > 0 { *dst = color; dst = dst.add(1); b -= 1; }
                    } else {
                        while b > 0 {
                            if x >= (*clip).clipped_pixels_left && x < ((*img).width - (*clip).clipped_pixels_right) {
                                *dst = color;
                            }
                            dst = dst.add(1);
                            x += 1;
                            b -= 1;
                        }
                    }
                }
            }
        }
    }
}

unsafe fn draw_compressed_and(img: *const Image, mut data: *const ColorT, x_offset: c_int, y_offset: c_int, height: c_int, color: ColorT) {
    unsafe {
        let clip = graphics_get_clip_info(x_offset, y_offset, (*img).width, height);
        if (*clip).is_visible == 0 { return; }
        let unclipped = (*clip).clip_x == ClipCode::None;
        
        for y in 0 .. (height - (*clip).clipped_pixels_bottom) {
            let mut x = 0;
            while x < (*img).width {
                let mut b = *data;
                data = data.add(1);
                if b == 255 {
                    x += *data as i32;
                    data = data.add(1);
                } else if y < (*clip).clipped_pixels_top {
                    data = data.add(b as usize);
                    x += b as i32;
                } else {
                    let mut pixels = data;
                    data = data.add(b as usize);
                    let mut dst = graphics_get_pixel(x_offset + x, y_offset + y);
                    if unclipped {
                        x += b as i32;
                        while b > 0 { *dst = *pixels & color; dst = dst.add(1); pixels = pixels.add(1); b -= 1; }
                    } else {
                        while b > 0 {
                            if x >= (*clip).clipped_pixels_left && x < ((*img).width - (*clip).clipped_pixels_right) {
                                *dst = *pixels & color;
                            }
                            dst = dst.add(1);
                            pixels = pixels.add(1);
                            x += 1;
                            b -= 1;
                        }
                    }
                }
            }
        }
    }
}

unsafe fn draw_compressed_blend(img: *const Image, mut data: *const ColorT, x_offset: c_int, y_offset: c_int, height: c_int, color: ColorT) {
    unsafe {
        let clip = graphics_get_clip_info(x_offset, y_offset, (*img).width, height);
        if (*clip).is_visible == 0 { return; }
        let unclipped = (*clip).clip_x == ClipCode::None;
        
        for y in 0 .. (height - (*clip).clipped_pixels_bottom) {
            let mut x = 0;
            while x < (*img).width {
                let mut b = *data;
                data = data.add(1);
                if b == 255 {
                    x += *data as i32;
                    data = data.add(1);
                } else if y < (*clip).clipped_pixels_top {
                    data = data.add(b as usize);
                    x += b as i32;
                } else {
                    data = data.add(b as usize);
                    let mut dst = graphics_get_pixel(x_offset + x, y_offset + y);
                    if unclipped {
                        x += b as i32;
                        while b > 0 { *dst &= color; dst = dst.add(1); b -= 1; }
                    } else {
                        while b > 0 {
                            if x >= (*clip).clipped_pixels_left && x < ((*img).width - (*clip).clipped_pixels_right) {
                                *dst &= color;
                            }
                            dst = dst.add(1);
                            x += 1;
                            b -= 1;
                        }
                    }
                }
            }
        }
    }
}

unsafe fn draw_compressed_blend_alpha(img: *const Image, mut data: *const ColorT, x_offset: c_int, y_offset: c_int, height: c_int, color: ColorT) {
    unsafe {
        let clip = graphics_get_clip_info(x_offset, y_offset, (*img).width, height);
        if (*clip).is_visible == 0 { return; }
        let alpha = component(color, 24);
        if alpha == 0 { return; }
        if alpha == 255 {
            draw_compressed_set(img, data, x_offset, y_offset, height, color);
            return;
        }
        let alpha_dst = 256 - alpha;
        let src_rb = (color & 0xff00ff) * alpha;
        let src_g = (color & 0x00ff00) * alpha;
        let unclipped = (*clip).clip_x == ClipCode::None;
        
        for y in 0 .. (height - (*clip).clipped_pixels_bottom) {
            let mut x = 0;
            let mut dst = graphics_get_pixel(x_offset, y_offset + y);
            while x < (*img).width {
                let mut b = *data;
                data = data.add(1);
                if b == 255 {
                    let skip = *data as i32;
                    x += skip;
                    dst = dst.add(skip as usize);
                    data = data.add(1);
                } else if y < (*clip).clipped_pixels_top {
                    data = data.add(b as usize);
                    x += b as i32;
                    dst = dst.add(b as usize);
                } else {
                    data = data.add(b as usize);
                    if unclipped {
                        x += b as i32;
                        while b > 0 {
                            let d = *dst;
                            *dst = (((src_rb + (d & 0xff00ff) * alpha_dst) & 0xff00ff00) |
                                    ((src_g  + (d & 0x00ff00) * alpha_dst) & 0x00ff0000)) >> 8;
                            b -= 1;
                            dst = dst.add(1);
                        }
                    } else {
                        while b > 0 {
                            if x >= (*clip).clipped_pixels_left && x < ((*img).width - (*clip).clipped_pixels_right) {
                                let d = *dst;
                                *dst = (((src_rb + (d & 0xff00ff) * alpha_dst) & 0xff00ff00) |
                                       ((src_g  + (d & 0x00ff00) * alpha_dst) & 0x00ff0000)) >> 8;
                            }
                            dst = dst.add(1);
                            x += 1;
                            b -= 1;
                        }
                    }
                }
            }
        }
    }
}

unsafe fn draw_footprint_simple(src: *const ColorT, x: i32, y: i32) {
    unsafe {
        let widths = [2, 6, 10, 14, 18, 22, 26, 30, 34, 38, 42, 46, 50, 54, 58, 58, 54, 50, 46, 42, 38, 34, 30, 26, 22, 18, 14, 10, 6, 2];
        for i in 0..30 {
            ptr::copy_nonoverlapping(
                src.add(FOOTPRINT_OFFSET_PER_HEIGHT[i] as usize),
                graphics_get_pixel(x + FOOTPRINT_X_START_PER_HEIGHT[i], y + i as i32),
                widths[i]
            );
        }
    }
}

unsafe fn draw_footprint_tile(data: *const ColorT, x_offset: i32, y_offset: i32, mut color_mask: ColorT) {
    unsafe {
        if color_mask == 0 { color_mask = 0xffffffff; }
        let clip = graphics_get_clip_info(x_offset, y_offset, FOOTPRINT_WIDTH, FOOTPRINT_HEIGHT);
        if (*clip).is_visible == 0 { return; }
        
        if (*clip).clip_y == ClipCode::None && (*clip).clip_x == ClipCode::None && color_mask == 0xffffffff {
            draw_footprint_simple(data, x_offset, y_offset);
            return;
        }
        
        let clip_left = (*clip).clip_x == ClipCode::Left || (*clip).clip_x == ClipCode::Both;
        let clip_right = (*clip).clip_x == ClipCode::Right || (*clip).clip_x == ClipCode::Both;
        let mut src = data.add(FOOTPRINT_OFFSET_PER_HEIGHT[(*clip).clipped_pixels_top as usize] as usize);
        
        for y in (*clip).clipped_pixels_top .. ((*clip).clipped_pixels_top + (*clip).visible_pixels_y) {
            let mut x_start = FOOTPRINT_X_START_PER_HEIGHT[y as usize];
            let mut x_max = 58 - x_start * 2;
            let mut x_pixel_advance = 0;
            
            if clip_left {
                if (*clip).clipped_pixels_left + (*clip).visible_pixels_x < x_start {
                    src = src.add(x_max as usize);
                    continue;
                }
                if (*clip).clipped_pixels_left > x_start {
                    let reduce = (*clip).clipped_pixels_left - x_start;
                    if reduce >= x_max {
                        src = src.add(x_max as usize);
                        continue;
                    }
                    src = src.add(reduce as usize);
                    x_max -= reduce;
                    x_start = (*clip).clipped_pixels_left;
                }
            }
            if clip_right {
                let cx = 58 - (*clip).clipped_pixels_right;
                if cx < x_start {
                    src = src.add(x_max as usize);
                    continue;
                }
                if x_start + x_max > cx {
                    let temp_max = cx - x_start;
                    x_pixel_advance = x_max - temp_max;
                    x_max = temp_max;
                }
            }
            
            let mut buffer = graphics_get_pixel(x_offset + x_start, y_offset + y);
            if color_mask == 0xffffffff {
                ptr::copy_nonoverlapping(src, buffer, x_max as usize);
                src = src.add((x_max + x_pixel_advance) as usize);
            } else {
                for _ in 0..x_max {
                    *buffer = *src & color_mask;
                    buffer = buffer.add(1);
                    src = src.add(1);
                }
                src = src.add(x_pixel_advance as usize);
            }
        }
    }
}

#[inline]
unsafe fn tile_data(data: *const ColorT, index: i32) -> *const ColorT {
    unsafe { data.add((900 * index) as usize) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_draw(image_id: c_int, x: c_int, y: c_int) {
    unsafe {
        let img = image_get(image_id);
        let data = image_data(image_id);
        if data.is_null() { return; }
        if (*img).draw.is_fully_compressed != 0 {
            draw_compressed(img, data, x, y, (*img).height);
        } else {
            draw_uncompressed(img, data, x, y, 0, DrawType::None);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_draw_enemy(image_id: c_int, x: c_int, y: c_int) {
    unsafe {
        if image_id <= 0 || image_id >= 801 { return; }
        let img = image_get_enemy(image_id);
        let data = image_data_enemy(image_id);
        if !data.is_null() {
            draw_compressed(img, data, x, y, (*img).height);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_draw_masked(image_id: c_int, x: c_int, y: c_int, color_mask: ColorT) {
    unsafe {
        let img = image_get(image_id);
        let data = image_data(image_id);
        if data.is_null() { return; }
        if (*img).draw.is_fully_compressed != 0 {
            if color_mask == 0 {
                draw_compressed(img, data, x, y, (*img).height);
            } else {
                draw_compressed_and(img, data, x, y, (*img).height, color_mask);
            }
        } else {
            draw_uncompressed(img, data, x, y, color_mask, if color_mask != 0 { DrawType::And } else { DrawType::None });
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_draw_blend(image_id: c_int, x: c_int, y: c_int, color: ColorT) {
    unsafe {
        let img = image_get(image_id);
        let data = image_data(image_id);
        if data.is_null() || (*img).draw.draw_type == IMAGE_TYPE_ISOMETRIC { return; }
        if (*img).draw.is_fully_compressed != 0 {
            draw_compressed_blend(img, data, x, y, (*img).height, color);
        } else {
            draw_uncompressed(img, data, x, y, color, DrawType::Blend);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_draw_blend_alpha(image_id: c_int, x: c_int, y: c_int, color: ColorT) {
    unsafe {
        let img = image_get(image_id);
        let data = image_data(image_id);
        if data.is_null() || (*img).draw.draw_type == IMAGE_TYPE_ISOMETRIC { return; }
        if (*img).draw.is_fully_compressed != 0 {
            draw_compressed_blend_alpha(img, data, x, y, (*img).height, color);
        } else {
            draw_uncompressed(img, data, x, y, color, DrawType::BlendAlpha);
        }
    }
}

unsafe fn draw_multibyte_letter(font: FontT, img: *const Image, data: *const ColorT, x: i32, y: i32, color: ColorT) {
    unsafe {
        match font {
            FontT::NormalWhite => {
                draw_uncompressed(img, data, x + 1, y + 1, 0x311c10, DrawType::BlendAlpha);
                draw_uncompressed(img, data, x, y, COLOR_WHITE, DrawType::BlendAlpha);
            }
            FontT::NormalRed => {
                draw_uncompressed(img, data, x + 1, y + 1, 0xe7cfad, DrawType::BlendAlpha);
                draw_uncompressed(img, data, x, y, 0x731408, DrawType::BlendAlpha);
            }
            FontT::NormalGreen => {
                draw_uncompressed(img, data, x + 1, y + 1, 0xe7cfad, DrawType::BlendAlpha);
                draw_uncompressed(img, data, x, y, 0x180800, DrawType::BlendAlpha);
            }
            FontT::NormalBlack | FontT::LargeBlack => {
                draw_uncompressed(img, data, x + 1, y + 1, 0xcead9c, DrawType::BlendAlpha);
                draw_uncompressed(img, data, x, y, COLOR_BLACK, DrawType::BlendAlpha);
            }
            _ => {
                draw_uncompressed(img, data, x, y, color, DrawType::BlendAlpha);
            }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_draw_letter(font: FontT, letter_id: c_int, x: c_int, y: c_int, color: ColorT) {
    unsafe {
        let img = image_letter(letter_id);
        let data = image_data_letter(letter_id);
        if data.is_null() { return; }
        if letter_id >= IMAGE_FONT_MULTIBYTE_OFFSET {
            draw_multibyte_letter(font, img, data, x, y, color);
            return;
        }
        if (*img).draw.is_fully_compressed != 0 {
            if color != 0 {
                draw_compressed_set(img, data, x, y, (*img).height, color);
            } else {
                draw_compressed(img, data, x, y, (*img).height);
            }
        } else {
            draw_uncompressed(img, data, x, y, color, if color != 0 { DrawType::Set } else { DrawType::None });
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_draw_fullscreen_background(image_id: c_int) {
    unsafe {
        let sw = screen_width();
        let sh = screen_height();
        if sw > 1024 || sh > 768 {
            graphics_clear_screen();
        }
        image_draw(image_id, (sw - 1024) / 2, (sh - 768) / 2);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_draw_isometric_footprint(image_id: c_int, x: c_int, y: c_int, color_mask: ColorT) {
    unsafe {
        let img = image_get(image_id);
        if (*img).draw.draw_type != IMAGE_TYPE_ISOMETRIC { return; }
        let data = image_data(image_id);
        match (*img).width {
            58 => draw_footprint_tile(tile_data(data, 0), x, y, color_mask),
            118 => {
                draw_footprint_tile(tile_data(data, 0), x, y, color_mask);
                draw_footprint_tile(tile_data(data, 1), x - 30, y + 15, color_mask);
                draw_footprint_tile(tile_data(data, 2), x + 30, y + 15, color_mask);
                draw_footprint_tile(tile_data(data, 3), x, y + 30, color_mask);
            }
            178 => {
                draw_footprint_tile(tile_data(data, 0), x, y, color_mask);
                draw_footprint_tile(tile_data(data, 1), x - 30, y + 15, color_mask);
                draw_footprint_tile(tile_data(data, 2), x + 30, y + 15, color_mask);
                draw_footprint_tile(tile_data(data, 3), x - 60, y + 30, color_mask);
                draw_footprint_tile(tile_data(data, 4), x, y + 30, color_mask);
                draw_footprint_tile(tile_data(data, 5), x + 60, y + 30, color_mask);
                draw_footprint_tile(tile_data(data, 6), x - 30, y + 45, color_mask);
                draw_footprint_tile(tile_data(data, 7), x + 30, y + 45, color_mask);
                draw_footprint_tile(tile_data(data, 8), x, y + 60, color_mask);
            }
            _ => {}
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_draw_isometric_footprint_from_draw_tile(image_id: c_int, x: c_int, y: c_int, color_mask: ColorT) {
    unsafe {
        let img = image_get(image_id);
        if (*img).draw.draw_type != IMAGE_TYPE_ISOMETRIC { return; }
        let data = image_data(image_id);
        match (*img).width {
            58 => draw_footprint_tile(tile_data(data, 0), x, y, color_mask),
            118 => {
                draw_footprint_tile(tile_data(data, 0), x + 30, y - 15, color_mask);
                draw_footprint_tile(tile_data(data, 1), x, y, color_mask);
                draw_footprint_tile(tile_data(data, 2), x + 60, y, color_mask);
                draw_footprint_tile(tile_data(data, 3), x + 30, y + 15, color_mask);
            }
            178 => {
                draw_footprint_tile(tile_data(data, 0), x + 60, y - 30, color_mask);
                draw_footprint_tile(tile_data(data, 1), x + 30, y - 15, color_mask);
                draw_footprint_tile(tile_data(data, 2), x + 90, y - 15, color_mask);
                draw_footprint_tile(tile_data(data, 3), x, y, color_mask);
                draw_footprint_tile(tile_data(data, 4), x + 60, y, color_mask);
                draw_footprint_tile(tile_data(data, 5), x + 120, y, color_mask);
                draw_footprint_tile(tile_data(data, 6), x + 30, y + 15, color_mask);
                draw_footprint_tile(tile_data(data, 7), x + 90, y + 15, color_mask);
                draw_footprint_tile(tile_data(data, 8), x + 60, y + 30, color_mask);
            }
            _ => {}
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_draw_isometric_top(image_id: c_int, mut x: c_int, mut y: c_int, color_mask: ColorT) {
    unsafe {
        let img = image_get(image_id);
        if (*img).draw.draw_type != IMAGE_TYPE_ISOMETRIC || (*img).draw.has_compressed_part == 0 { return; }
        let data = image_data(image_id).add((*img).draw.uncompressed_length as usize);
        let mut height = (*img).height;
        match (*img).width {
            58 => { y -= (*img).height - 30; height -= 16; }
            118 => { x -= 30; y -= (*img).height - 60; height -= 31; }
            178 => { x -= 60; y -= (*img).height - 90; height -= 46; }
            238 => { x -= 90; y -= (*img).height - 120; height -= 61; }
            298 => { x -= 120; y -= (*img).height - 150; height -= 76; }
            _ => {}
        }
        if color_mask == 0 {
            draw_compressed(img, data, x, y, height);
        } else {
            draw_compressed_and(img, data, x, y, height, color_mask);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_draw_isometric_top_from_draw_tile(image_id: c_int, x: c_int, mut y: c_int, color_mask: ColorT) {
    unsafe {
        let img = image_get(image_id);
        if (*img).draw.draw_type != IMAGE_TYPE_ISOMETRIC || (*img).draw.has_compressed_part == 0 { return; }
        let data = image_data(image_id).add((*img).draw.uncompressed_length as usize);
        let mut height = (*img).height;
        match (*img).width {
            58 => { y -= (*img).height - 30; height -= 16; }
            118 => { y -= (*img).height - 45; height -= 31; }
            178 => { y -= (*img).height - 60; height -= 46; }
            238 => { y -= (*img).height - 75; height -= 61; }
            298 => { y -= (*img).height - 90; height -= 76; }
            _ => {}
        }
        if color_mask == 0 {
            draw_compressed(img, data, x, y, height);
        } else {
            draw_compressed_and(img, data, x, y, height, color_mask);
        }
    }
}

unsafe fn color_average(img: *const Image, data: *const ColorT, mut x: i32, mut y: i32, scale_factor: u32) -> ColorT {
    unsafe {
        x *= scale_factor as i32;
        y *= scale_factor as i32;
        let mut rb = 0usize;
        let mut g = 0usize;
        let mut num_colors = 0usize;
        let mut num_transparent = 0usize;
        let max_x = x + scale_factor as i32;
        let max_y = y + scale_factor as i32;
        
        let mut yy = y;
        while yy < max_y {
            if yy >= (*img).height { break; }
            let mut xx = x;
            while xx < max_x {
                if xx >= (*img).width { break; }
                let color = *data.add((yy * (*img).width + xx) as usize);
                if color == COLOR_SG2_TRANSPARENT {
                    num_transparent += 1;
                } else {
                    rb += (color & 0xff00ff) as usize;
                    g += (color & 0xff00) as usize;
                    num_colors += 1;
                }
                xx += 1;
            }
            yy += 1;
        }
        if num_transparent > num_colors || num_colors == 0 {
            COLOR_SG2_TRANSPARENT
        } else {
            (( (rb / num_colors) & 0xff0000 ) | ( (g / num_colors) & 0xff00 ) | ( (rb / num_colors) & 0xffff )) as ColorT
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_draw_scaled_down(image_id: c_int, x_offset: c_int, y_offset: c_int, scale_factor: u32) {
    unsafe {
        let img = image_get(image_id);
        let data = image_data(image_id);
        if data.is_null() || (*img).draw.draw_type == IMAGE_TYPE_ISOMETRIC || (*img).draw.is_fully_compressed != 0 || scale_factor == 0 {
            return;
        }
        let width = (*img).width / scale_factor as i32;
        let height = (*img).height / scale_factor as i32;
        if width == 0 || height == 0 { return; }
        
        let clip = graphics_get_clip_info(x_offset, y_offset, width, height);
        if (*clip).is_visible == 0 { return; }
        
        for y in (*clip).clipped_pixels_top .. (height - (*clip).clipped_pixels_bottom) {
            let mut dst = graphics_get_pixel(x_offset + (*clip).clipped_pixels_left, y_offset + y);
            for x in (*clip).clipped_pixels_left .. (width - (*clip).clipped_pixels_right) {
                *dst = color_average(img, data, x, y, scale_factor);
                dst = dst.add(1);
            }
        }
    }
}
