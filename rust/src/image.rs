use std::ffi::{c_char, c_int};
use std::ptr::{self, addr_of, addr_of_mut};
use crate::buffer::{Buffer, buffer_init, buffer_set, buffer_skip, buffer_read_u8, buffer_read_i8, buffer_read_u16, buffer_read_i16, buffer_read_i32, buffer_read_raw};

pub type ColorT = u32;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ImageDraw {
    pub draw_type: c_int,
    pub is_fully_compressed: c_int,
    pub is_external: c_int,
    pub has_compressed_part: c_int,
    pub bitmap_id: c_int,
    pub offset: c_int,
    pub data_length: c_int,
    pub uncompressed_length: c_int,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Image {
    pub width: c_int,
    pub height: c_int,
    pub num_animation_sprites: c_int,
    pub sprite_offset_x: c_int,
    pub sprite_offset_y: c_int,
    pub animation_can_reverse: c_int,
    pub animation_speed_id: c_int,
    pub draw: ImageDraw,
}

const MAIN_ENTRIES: usize = 10000;
const ENEMY_ENTRIES: usize = 801;
const HEADER_SIZE: usize = 20680;
const ENTRY_SIZE: usize = 64;
const MAIN_INDEX_SIZE: usize = 660680;
const ENEMY_INDEX_OFFSET: usize = HEADER_SIZE;
const ENEMY_INDEX_SIZE: usize = ENTRY_SIZE * ENEMY_ENTRIES;
const EXTERNAL_FONT_ENTRIES: usize = 2000;
const EXTERNAL_FONT_INDEX_OFFSET: usize = HEADER_SIZE;
const EXTERNAL_FONT_INDEX_SIZE: usize = ENTRY_SIZE * EXTERNAL_FONT_ENTRIES;

const MAIN_DATA_SIZE: usize = 30000000;
const EMPIRE_DATA_SIZE: usize = 2000 * 1000 * 4;
const ENEMY_DATA_SIZE: usize = 2400000;
const EXTERNAL_FONT_DATA_SIZE: usize = 1500000;
const SCRATCH_DATA_SIZE: usize = 12100000;

const FULL_CHARSET_IN_FONT: c_int = 1;
const MULTIBYTE_IN_FONT: c_int = 2;

const IMAGE_FONT_MULTIBYTE_OFFSET: c_int = 10000;

const GROUP_FONT: usize = 16;
const GROUP_BUILDING_FOUNTAIN_4: usize = 53;
const GROUP_BUILDING_FOUNTAIN_3: usize = 56;
const GROUP_BUILDING_LION_HOUSE: usize = 50;
const GROUP_BUILDING_ENGINEERS_POST: usize = 81;
const GROUP_EMPIRE_MAP: usize = 47;

struct ImageData {
    current_climate: c_int,
    is_editor: c_int,
    fonts_enabled: c_int,
    font_base_offset: c_int,
    group_image_ids: [u16; 300],
    bitmaps: [[c_char; 200]; 100],
    main: [Image; MAIN_ENTRIES],
    enemy: [Image; ENEMY_ENTRIES],
    font: *mut Image,
    main_data: *mut ColorT,
    empire_data: *mut ColorT,
    enemy_data: *mut ColorT,
    font_data: *mut ColorT,
    tmp_data: *mut u8,
}

static mut DATA: ImageData = ImageData {
    current_climate: -1,
    is_editor: 0,
    fonts_enabled: 0,
    font_base_offset: 0,
    group_image_ids: [0; 300],
    bitmaps: [[0; 200]; 100],
    main: [unsafe { std::mem::zeroed() }; MAIN_ENTRIES],
    enemy: [unsafe { std::mem::zeroed() }; ENEMY_ENTRIES],
    font: ptr::null_mut(),
    main_data: ptr::null_mut(),
    empire_data: ptr::null_mut(),
    enemy_data: ptr::null_mut(),
    font_data: ptr::null_mut(),
    tmp_data: ptr::null_mut(),
};

static DUMMY_IMAGE: Image = unsafe { std::mem::zeroed() };

unsafe extern "C" {
    fn io_read_file_into_buffer(filepath: *const c_char, localizable: c_int, buffer: *mut std::ffi::c_void, max_size: c_int) -> c_int;
    fn io_read_file_part_into_buffer(filepath: *const c_char, localizable: c_int, buffer: *mut std::ffi::c_void, size: c_int, offset_in_file: c_int) -> c_int;
    fn log_error(msg: *const c_char, param_str: *const c_char, param_int: c_int);
}

const MAY_BE_LOCALIZED: c_int = 1;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_init() -> c_int {
    unsafe {
        (*addr_of_mut!(DATA.enemy_data)) = libc::malloc(ENEMY_DATA_SIZE) as *mut ColorT;
        (*addr_of_mut!(DATA.main_data)) = libc::malloc(MAIN_DATA_SIZE) as *mut ColorT;
        (*addr_of_mut!(DATA.empire_data)) = libc::malloc(EMPIRE_DATA_SIZE) as *mut ColorT;
        (*addr_of_mut!(DATA.tmp_data)) = libc::malloc(SCRATCH_DATA_SIZE) as *mut u8;
        
        if (*addr_of!(DATA.main_data)).is_null() || (*addr_of!(DATA.empire_data)).is_null() || 
           (*addr_of!(DATA.enemy_data)).is_null() || (*addr_of!(DATA.tmp_data)).is_null() {
            libc::free((*addr_of!(DATA.main_data)) as *mut libc::c_void);
            libc::free((*addr_of!(DATA.empire_data)) as *mut libc::c_void);
            libc::free((*addr_of!(DATA.enemy_data)) as *mut libc::c_void);
            libc::free((*addr_of!(DATA.tmp_data)) as *mut libc::c_void);
            return 0;
        }
        1
    }
}

unsafe fn prepare_index(images: *mut Image, size: c_int) {
    unsafe {
        let mut offset = 4;
        for i in 1..size as usize {
            let img = images.add(i);
            if (*img).draw.is_external != 0 {
                if (*img).draw.offset == 0 {
                    (*img).draw.offset = 1;
                }
            } else {
                (*img).draw.offset = offset;
                offset += (*img).draw.data_length;
            }
        }
    }
}

unsafe fn read_index_entry(buf: *mut Buffer, img: *mut Image) {
    unsafe {
        (*img).draw.offset = buffer_read_i32(buf);
        (*img).draw.data_length = buffer_read_i32(buf);
        (*img).draw.uncompressed_length = buffer_read_i32(buf);
        buffer_skip(buf, 8);
        (*img).width = buffer_read_u16(buf) as c_int;
        (*img).height = buffer_read_u16(buf) as c_int;
        buffer_skip(buf, 6);
        (*img).num_animation_sprites = buffer_read_u16(buf) as c_int;
        buffer_skip(buf, 2);
        (*img).sprite_offset_x = buffer_read_i16(buf) as c_int;
        (*img).sprite_offset_y = buffer_read_i16(buf) as c_int;
        buffer_skip(buf, 10);
        (*img).animation_can_reverse = buffer_read_i8(buf) as c_int;
        buffer_skip(buf, 1);
        (*img).draw.draw_type = buffer_read_u8(buf) as c_int;
        (*img).draw.is_fully_compressed = buffer_read_i8(buf) as c_int;
        (*img).draw.is_external = buffer_read_i8(buf) as c_int;
        (*img).draw.has_compressed_part = buffer_read_i8(buf) as c_int;
        buffer_skip(buf, 2);
        (*img).draw.bitmap_id = buffer_read_u8(buf) as c_int;
        buffer_skip(buf, 1);
        (*img).animation_speed_id = buffer_read_u8(buf) as c_int;
        buffer_skip(buf, 5);
    }
}

unsafe fn read_index(buf: *mut Buffer, images: *mut Image, size: c_int) {
    unsafe {
        for i in 0..size as usize {
            read_index_entry(buf, images.add(i));
        }
        prepare_index(images, size);
    }
}

unsafe fn read_header(buf: *mut Buffer) {
    unsafe {
        buffer_skip(buf, 80);
        for i in 0..300 {
            (*addr_of_mut!(DATA.group_image_ids))[i] = buffer_read_u16(buf);
        }
        buffer_read_raw(buf, addr_of_mut!(DATA.bitmaps) as *mut std::ffi::c_void, 20000);
    }
}

#[inline]
fn to_32_bit(c: u16) -> ColorT {
    ((c as u32 & 0x7c00) << 9) | ((c as u32 & 0x7000) << 4) |
    ((c as u32 & 0x3e0) << 6)  | ((c as u32 & 0x380) << 1) |
    ((c as u32 & 0x1f) << 3)   | ((c as u32 & 0x1c) >> 2)
}

unsafe fn convert_uncompressed(buf: *mut Buffer, buf_length: c_int, mut dst: *mut ColorT) -> c_int {
    unsafe {
        for _ in (0..buf_length).step_by(2) {
            *dst = to_32_bit(buffer_read_u16(buf));
            dst = dst.add(1);
        }
        buf_length / 2
    }
}

unsafe fn convert_compressed(buf: *mut Buffer, mut buf_length: c_int, mut dst: *mut ColorT) -> c_int {
    unsafe {
        let mut dst_length = 0;
        while buf_length > 0 {
            let control = buffer_read_u8(buf);
            if control == 255 {
                *dst = 255; dst = dst.add(1);
                *dst = buffer_read_u8(buf) as ColorT; dst = dst.add(1);
                dst_length += 2;
                buf_length -= 2;
            } else {
                *dst = control as ColorT; dst = dst.add(1);
                for _ in 0..control {
                    *dst = to_32_bit(buffer_read_u16(buf));
                    dst = dst.add(1);
                }
                dst_length += control as c_int + 1;
                buf_length -= (control as c_int * 2) + 1;
            }
        }
        dst_length
    }
}

unsafe fn convert_images(images: *mut Image, size: c_int, buf: *mut Buffer, start_dst: *mut ColorT) {
    unsafe {
        let mut dst = start_dst.add(1);
        for i in 0..size as usize {
            let img = images.add(i);
            if (*img).draw.is_external != 0 {
                continue;
            }
            buffer_set(buf, (*img).draw.offset);
            let img_offset = (dst as usize - start_dst as usize) / std::mem::size_of::<ColorT>();
            if (*img).draw.is_fully_compressed != 0 {
                dst = dst.add(convert_compressed(buf, (*img).draw.data_length, dst) as usize);
            } else if (*img).draw.has_compressed_part != 0 {
                dst = dst.add(convert_uncompressed(buf, (*img).draw.uncompressed_length, dst) as usize);
                dst = dst.add(convert_compressed(buf, (*img).draw.data_length - (*img).draw.uncompressed_length, dst) as usize);
            } else {
                dst = dst.add(convert_uncompressed(buf, (*img).draw.data_length, dst) as usize);
            }
            (*img).draw.offset = img_offset as c_int;
            (*img).draw.uncompressed_length /= 2;
        }
    }
}

unsafe fn load_empire() {
    unsafe {
        let empire_555 = "The_empire.555\0".as_ptr() as *const c_char;
        let size = io_read_file_into_buffer(empire_555, MAY_BE_LOCALIZED, (*addr_of!(DATA.tmp_data)) as *mut std::ffi::c_void, EMPIRE_DATA_SIZE as i32);
        if size != (EMPIRE_DATA_SIZE / 2) as i32 {
            log_error("unable to load empire data\0".as_ptr() as *const c_char, empire_555, 0);
            return;
        }
        let mut buf = std::mem::zeroed();
        buffer_init(&mut buf, (*addr_of!(DATA.tmp_data)) as *mut std::ffi::c_void, size);
        convert_uncompressed(&mut buf, size, *addr_of!(DATA.empire_data));
    }
}

unsafe fn fix_animation_offsets() {
    unsafe {
        let ids = addr_of!(DATA.group_image_ids);
        let main = addr_of_mut!(DATA.main);
        (*main)[(*ids)[GROUP_BUILDING_FOUNTAIN_4] as usize].sprite_offset_x -= 1;
        (*main)[(*ids)[GROUP_BUILDING_FOUNTAIN_3] as usize].sprite_offset_x -= 1;
        (*main)[(*ids)[GROUP_BUILDING_LION_HOUSE] as usize].sprite_offset_y -= 1;
        (*main)[(*ids)[GROUP_BUILDING_ENGINEERS_POST] as usize].sprite_offset_y += 1;
    }
}

#[repr(transparent)]
struct SyncPtr(*const c_char);
unsafe impl Sync for SyncPtr {}

static MAIN_GRAPHICS_SG2: [SyncPtr; 3] = [
    SyncPtr("c3.sg2\0".as_ptr() as *const c_char),
    SyncPtr("c3_north.sg2\0".as_ptr() as *const c_char),
    SyncPtr("c3_south.sg2\0".as_ptr() as *const c_char),
];
static MAIN_GRAPHICS_555: [SyncPtr; 3] = [
    SyncPtr("c3.555\0".as_ptr() as *const c_char),
    SyncPtr("c3_north.555\0".as_ptr() as *const c_char),
    SyncPtr("c3_south.555\0".as_ptr() as *const c_char),
];
static EDITOR_GRAPHICS_SG2: [SyncPtr; 3] = [
    SyncPtr("c3map.sg2\0".as_ptr() as *const c_char),
    SyncPtr("c3map_north.sg2\0".as_ptr() as *const c_char),
    SyncPtr("c3map_south.sg2\0".as_ptr() as *const c_char),
];
static EDITOR_GRAPHICS_555: [SyncPtr; 3] = [
    SyncPtr("c3map.555\0".as_ptr() as *const c_char),
    SyncPtr("c3map_north.555\0".as_ptr() as *const c_char),
    SyncPtr("c3map_south.555\0".as_ptr() as *const c_char),
];

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_load_climate(climate_id: c_int, is_editor: c_int, force_reload: c_int) -> c_int {
    unsafe {
        if climate_id == *addr_of!(DATA.current_climate) && is_editor == *addr_of!(DATA.is_editor) && force_reload == 0 {
            return 1;
        }
        let idx = climate_id as usize;
        let filename_bmp = if is_editor != 0 { EDITOR_GRAPHICS_555[idx].0 } else { MAIN_GRAPHICS_555[idx].0 };
        let filename_idx = if is_editor != 0 { EDITOR_GRAPHICS_SG2[idx].0 } else { MAIN_GRAPHICS_SG2[idx].0 };

        if MAIN_INDEX_SIZE as i32 != io_read_file_into_buffer(filename_idx, MAY_BE_LOCALIZED, (*addr_of!(DATA.tmp_data)) as *mut std::ffi::c_void, MAIN_INDEX_SIZE as i32) {
            return 0;
        }

        let mut buf = std::mem::zeroed();
        buffer_init(&mut buf, (*addr_of!(DATA.tmp_data)) as *mut std::ffi::c_void, HEADER_SIZE as i32);
        read_header(&mut buf);
        buffer_init(&mut buf, (*addr_of!(DATA.tmp_data)).add(HEADER_SIZE) as *mut std::ffi::c_void, (ENTRY_SIZE * MAIN_ENTRIES) as i32);
        read_index(&mut buf, addr_of_mut!(DATA.main) as *mut Image, MAIN_ENTRIES as i32);

        let data_size = io_read_file_into_buffer(filename_bmp, MAY_BE_LOCALIZED, (*addr_of!(DATA.tmp_data)) as *mut std::ffi::c_void, SCRATCH_DATA_SIZE as i32);
        if data_size == 0 {
            return 0;
        }
        buffer_init(&mut buf, (*addr_of!(DATA.tmp_data)) as *mut std::ffi::c_void, data_size);
        convert_images(addr_of_mut!(DATA.main) as *mut Image, MAIN_ENTRIES as i32, &mut buf, *addr_of!(DATA.main_data));
        
        (*addr_of_mut!(DATA.current_climate)) = climate_id;
        (*addr_of_mut!(DATA.is_editor)) = is_editor;

        load_empire();
        if is_editor == 0 {
            fix_animation_offsets();
        }
        1
    }
}

unsafe fn free_font_memory() {
    unsafe {
        libc::free((*addr_of!(DATA.font)) as *mut libc::c_void);
        libc::free((*addr_of!(DATA.font_data)) as *mut libc::c_void);
        (*addr_of_mut!(DATA.font)) = ptr::null_mut();
        (*addr_of_mut!(DATA.font_data)) = ptr::null_mut();
        (*addr_of_mut!(DATA.fonts_enabled)) = 0; // replaced NO_EXTRA_FONT constant
    }
}

unsafe fn alloc_font_memory(font_entries: c_int, font_data_size: usize) -> bool {
    unsafe {
        free_font_memory();
        (*addr_of_mut!(DATA.font)) = libc::malloc(font_entries as usize * std::mem::size_of::<Image>()) as *mut Image;
        (*addr_of_mut!(DATA.font_data)) = libc::malloc(font_data_size) as *mut ColorT;
        if (*addr_of!(DATA.font)).is_null() || (*addr_of!(DATA.font_data)).is_null() {
            libc::free((*addr_of!(DATA.font)) as *mut libc::c_void);
            libc::free((*addr_of!(DATA.font_data)) as *mut libc::c_void);
            (*addr_of_mut!(DATA.font)) = ptr::null_mut();
            (*addr_of_mut!(DATA.font_data)) = ptr::null_mut();
            return false;
        }
        ptr::write_bytes(*addr_of!(DATA.font), 0, font_entries as usize);
        true
    }
}

static ENEMY_GRAPHICS_SG2: [SyncPtr; 20] = [
    SyncPtr("goths.sg2\0".as_ptr() as *const c_char),
    SyncPtr("Etruscan.sg2\0".as_ptr() as *const c_char),
    SyncPtr("Etruscan.sg2\0".as_ptr() as *const c_char),
    SyncPtr("carthage.sg2\0".as_ptr() as *const c_char),
    SyncPtr("Greek.sg2\0".as_ptr() as *const c_char),
    SyncPtr("Greek.sg2\0".as_ptr() as *const c_char),
    SyncPtr("egyptians.sg2\0".as_ptr() as *const c_char),
    SyncPtr("Persians.sg2\0".as_ptr() as *const c_char),
    SyncPtr("Phoenician.sg2\0".as_ptr() as *const c_char),
    SyncPtr("celts.sg2\0".as_ptr() as *const c_char),
    SyncPtr("celts.sg2\0".as_ptr() as *const c_char),
    SyncPtr("celts.sg2\0".as_ptr() as *const c_char),
    SyncPtr("Gaul.sg2\0".as_ptr() as *const c_char),
    SyncPtr("Gaul.sg2\0".as_ptr() as *const c_char),
    SyncPtr("goths.sg2\0".as_ptr() as *const c_char),
    SyncPtr("goths.sg2\0".as_ptr() as *const c_char),
    SyncPtr("goths.sg2\0".as_ptr() as *const c_char),
    SyncPtr("Phoenician.sg2\0".as_ptr() as *const c_char),
    SyncPtr("North African.sg2\0".as_ptr() as *const c_char),
    SyncPtr("Phoenician.sg2\0".as_ptr() as *const c_char),
];
static ENEMY_GRAPHICS_555: [SyncPtr; 20] = [
    SyncPtr("goths.555\0".as_ptr() as *const c_char),
    SyncPtr("Etruscan.555\0".as_ptr() as *const c_char),
    SyncPtr("Etruscan.555\0".as_ptr() as *const c_char),
    SyncPtr("carthage.555\0".as_ptr() as *const c_char),
    SyncPtr("Greek.555\0".as_ptr() as *const c_char),
    SyncPtr("Greek.555\0".as_ptr() as *const c_char),
    SyncPtr("egyptians.555\0".as_ptr() as *const c_char),
    SyncPtr("Persians.555\0".as_ptr() as *const c_char),
    SyncPtr("Phoenician.555\0".as_ptr() as *const c_char),
    SyncPtr("celts.555\0".as_ptr() as *const c_char),
    SyncPtr("celts.555\0".as_ptr() as *const c_char),
    SyncPtr("celts.555\0".as_ptr() as *const c_char),
    SyncPtr("Gaul.555\0".as_ptr() as *const c_char),
    SyncPtr("Gaul.555\0".as_ptr() as *const c_char),
    SyncPtr("goths.555\0".as_ptr() as *const c_char),
    SyncPtr("goths.555\0".as_ptr() as *const c_char),
    SyncPtr("goths.555\0".as_ptr() as *const c_char),
    SyncPtr("Phoenician.555\0".as_ptr() as *const c_char),
    SyncPtr("North African.555\0".as_ptr() as *const c_char),
    SyncPtr("Phoenician.555\0".as_ptr() as *const c_char),
];

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_load_enemy(enemy_id: c_int) -> c_int {
    unsafe {
        let idx = enemy_id as usize;
        let filename_bmp = ENEMY_GRAPHICS_555[idx].0;
        let filename_idx = ENEMY_GRAPHICS_SG2[idx].0;

        if ENEMY_INDEX_SIZE as i32 != io_read_file_part_into_buffer(filename_idx, MAY_BE_LOCALIZED, (*addr_of!(DATA.tmp_data)) as *mut std::ffi::c_void, ENEMY_INDEX_SIZE as i32, ENEMY_INDEX_OFFSET as i32) {
            return 0;
        }

        let mut buf = std::mem::zeroed();
        buffer_init(&mut buf, (*addr_of!(DATA.tmp_data)) as *mut std::ffi::c_void, ENEMY_INDEX_SIZE as i32);
        read_index(&mut buf, addr_of_mut!(DATA.enemy) as *mut Image, ENEMY_ENTRIES as i32);

        let data_size = io_read_file_into_buffer(filename_bmp, MAY_BE_LOCALIZED, (*addr_of!(DATA.tmp_data)) as *mut std::ffi::c_void, SCRATCH_DATA_SIZE as i32);
        if data_size == 0 {
            return 0;
        }
        buffer_init(&mut buf, (*addr_of!(DATA.tmp_data)) as *mut std::ffi::c_void, data_size);
        convert_images(addr_of_mut!(DATA.enemy) as *mut Image, ENEMY_ENTRIES as i32, &mut buf, *addr_of!(DATA.enemy_data));
        1
    }
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EncodingType {
    EncodingIso8859_1 = 0,
    EncodingCyrillic = 1,
    EncodingGreek = 2,
    EncodingSimplifiedChinese = 3,
    EncodingTraditionalChinese = 4,
    EncodingKorean = 5,
    EncodingJapanese = 6,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_load_fonts(encoding: EncodingType) -> c_int {
    unsafe {
        match encoding {
            EncodingType::EncodingCyrillic => load_external_fonts(201),
            EncodingType::EncodingGreek => load_external_fonts(1),
            EncodingType::EncodingTraditionalChinese => load_multibyte_fonts(EncodingType::EncodingTraditionalChinese),
            EncodingType::EncodingSimplifiedChinese => load_multibyte_fonts(EncodingType::EncodingSimplifiedChinese),
            EncodingType::EncodingKorean => load_multibyte_fonts(EncodingType::EncodingKorean),
            EncodingType::EncodingJapanese => load_multibyte_fonts(EncodingType::EncodingJapanese),
            _ => {
                free_font_memory();
                1
            }
        }
    }
}

unsafe fn load_external_fonts(base_offset: c_int) -> c_int {
    unsafe {
        if !alloc_font_memory(EXTERNAL_FONT_ENTRIES as c_int, EXTERNAL_FONT_DATA_SIZE) {
            return 0;
        }
        let sg2 = "C3_fonts.sg2\0".as_ptr() as *const c_char;
        if EXTERNAL_FONT_INDEX_SIZE as i32 != io_read_file_part_into_buffer(sg2, MAY_BE_LOCALIZED, (*addr_of!(DATA.tmp_data)) as *mut std::ffi::c_void, EXTERNAL_FONT_INDEX_SIZE as i32, EXTERNAL_FONT_INDEX_OFFSET as i32) {
            return 0;
        }
        let mut buf = std::mem::zeroed();
        buffer_init(&mut buf, (*addr_of!(DATA.tmp_data)) as *mut std::ffi::c_void, EXTERNAL_FONT_INDEX_SIZE as i32);
        read_index(&mut buf, *addr_of!(DATA.font), EXTERNAL_FONT_ENTRIES as i32);

        let bmp = "C3_fonts.555\0".as_ptr() as *const c_char;
        let data_size = io_read_file_into_buffer(bmp, MAY_BE_LOCALIZED, (*addr_of!(DATA.tmp_data)) as *mut std::ffi::c_void, SCRATCH_DATA_SIZE as i32);
        if data_size == 0 {
            return 0;
        }
        buffer_init(&mut buf, (*addr_of!(DATA.tmp_data)) as *mut std::ffi::c_void, data_size);
        convert_images(*addr_of!(DATA.font), EXTERNAL_FONT_ENTRIES as i32, &mut buf, *addr_of!(DATA.font_data));

        (*addr_of_mut!(DATA.fonts_enabled)) = FULL_CHARSET_IN_FONT;
        (*addr_of_mut!(DATA.font_base_offset)) = base_offset;
        1
    }
}

// Stub for multibyte font loading, needs actual logic from image.c
unsafe fn load_multibyte_fonts(_encoding: EncodingType) -> c_int {
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_group(group: c_int) -> c_int {
    unsafe { (*addr_of!(DATA.group_image_ids))[group as usize] as c_int }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_get(id: c_int) -> *const Image {
    unsafe {
        if id >= 0 && id < MAIN_ENTRIES as i32 {
            (addr_of!(DATA.main) as *const Image).add(id as usize)
        } else {
            ptr::null()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_letter(letter_id: c_int) -> *const Image {
    unsafe {
        if *addr_of!(DATA.fonts_enabled) == FULL_CHARSET_IN_FONT {
            (*addr_of!(DATA.font)).add((*addr_of!(DATA.font_base_offset)) as usize + letter_id as usize)
        } else if *addr_of!(DATA.fonts_enabled) == MULTIBYTE_IN_FONT && letter_id >= IMAGE_FONT_MULTIBYTE_OFFSET {
            (*addr_of!(DATA.font)).add((*addr_of!(DATA.font_base_offset)) as usize + (letter_id - IMAGE_FONT_MULTIBYTE_OFFSET) as usize)
        } else if letter_id < IMAGE_FONT_MULTIBYTE_OFFSET {
            let ids = addr_of!(DATA.group_image_ids);
            (addr_of!(DATA.main) as *const Image).add(((*ids)[GROUP_FONT] as i32 + letter_id) as usize)
        } else {
            addr_of!(DUMMY_IMAGE)
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_get_enemy(id: c_int) -> *const Image {
    unsafe {
        if id >= 0 && id < ENEMY_ENTRIES as i32 {
            (addr_of!(DATA.enemy) as *const Image).add(id as usize)
        } else {
            ptr::null()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_data(id: c_int) -> *const ColorT {
    unsafe {
        if id < 0 || id >= MAIN_ENTRIES as i32 {
            return ptr::null();
        }
        let img = (addr_of!(DATA.main) as *const Image).add(id as usize);
        if (*img).draw.is_external == 0 {
            (*addr_of!(DATA.main_data)).add((*img).draw.offset as usize)
        } else if id == image_group(GROUP_EMPIRE_MAP as i32) {
            *addr_of!(DATA.empire_data)
        } else {
            ptr::null()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_data_letter(letter_id: c_int) -> *const ColorT {
    unsafe {
        if *addr_of!(DATA.fonts_enabled) == FULL_CHARSET_IN_FONT {
            let img = (*addr_of!(DATA.font)).add((*addr_of!(DATA.font_base_offset)) as usize + letter_id as usize);
            (*addr_of!(DATA.font_data)).add((*img).draw.offset as usize)
        } else if *addr_of!(DATA.fonts_enabled) == MULTIBYTE_IN_FONT && letter_id >= IMAGE_FONT_MULTIBYTE_OFFSET {
            let img = (*addr_of!(DATA.font)).add((*addr_of!(DATA.font_base_offset)) as usize + (letter_id - IMAGE_FONT_MULTIBYTE_OFFSET) as usize);
            (*addr_of!(DATA.font_data)).add((*img).draw.offset as usize)
        } else if letter_id < IMAGE_FONT_MULTIBYTE_OFFSET {
            let ids = addr_of!(DATA.group_image_ids);
            let image_id = (*ids)[GROUP_FONT] as i32 + letter_id;
            let img = (addr_of!(DATA.main) as *const Image).add(image_id as usize);
            (*addr_of!(DATA.main_data)).add((*img).draw.offset as usize)
        } else {
            ptr::null()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_data_enemy(id: c_int) -> *const ColorT {
    unsafe {
        let img = (addr_of!(DATA.enemy) as *const Image).add(id as usize);
        if (*img).draw.offset > 0 {
            (*addr_of!(DATA.enemy_data)).add((*img).draw.offset as usize)
        } else {
            ptr::null()
        }
    }
}
