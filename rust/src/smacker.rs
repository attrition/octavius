use std::ffi::{c_int, c_void, c_long};
use std::ptr::{self, addr_of};
use crate::image::ColorT;

// Smacker constants
const MAX_TRACKS: usize = 7;
const MAX_PALETTE: usize = 256;
const HEADER_SIZE: usize = 104;

const FLAG_Y_INTERLACE: i32 = 0x02;
const FLAG_Y_DOUBLE: i32 = 0x04;

const AUDIO_FLAG_STEREO: i32 = 0x10000000;
const AUDIO_FLAG_16BIT: i32 = 0x20000000;
const AUDIO_FLAG_HAS_TRACK: i32 = 0x40000000;
const AUDIO_FLAG_COMPRESSED: i32 = 0x80000000u32 as i32;

const BLOCK_MONO: i32 = 0;
const BLOCK_FULL: i32 = 1;
const BLOCK_SOLID: i32 = 3;

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SmackerFrameStatus {
    Error = 0,
    Ok = 1,
    Done = 2,
}

struct Bitstream<'a> {
    data: &'a [u8],
    index: usize,
    bit_index: i32,
}

impl<'a> Bitstream<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, index: 0, bit_index: 0 }
    }

    fn read_bit(&mut self) -> i32 {
        if self.index >= self.data.len() {
            return 0;
        }
        let result = (self.data[self.index] & (1 << self.bit_index)) != 0;
        self.bit_index += 1;
        if self.bit_index >= 8 {
            self.index += 1;
            self.bit_index = 0;
        }
        result as i32
    }

    fn read_byte(&mut self) -> u8 {
        if self.bit_index == 0 {
            if self.index < self.data.len() {
                let val = self.data[self.index];
                self.index += 1;
                return val;
            }
            return 0;
        }
        if self.index + 1 >= self.data.len() {
            return 0;
        }
        let masks = [0x00, 0x01, 0x03, 0x07, 0x0f, 0x1f, 0x3f, 0x7f, 0xff];
        let mut value = self.data[self.index] >> self.bit_index;
        self.index += 1;
        value |= (self.data[self.index] & masks[self.bit_index as usize]) << (8 - self.bit_index);
        value
    }
}

enum HuffNode8 {
    Branch(Box<[HuffNode8; 2]>),
    Leaf(u8),
}

struct HuffTree8 {
    root: Option<HuffNode8>,
}

impl HuffTree8 {
    fn build(bs: &mut Bitstream) -> Option<HuffNode8> {
        if bs.read_bit() != 0 {
            let left = Self::build(bs)?;
            let right = Self::build(bs)?;
            Some(HuffNode8::Branch(Box::new([left, right])))
        } else {
            Some(HuffNode8::Leaf(bs.read_byte()))
        }
    }

    fn create(bs: &mut Bitstream) -> Option<Self> {
        if bs.read_bit() != 0 {
            let root = Self::build(bs)?;
            if bs.read_bit() != 0 {
                return None;
            }
            Some(Self { root: Some(root) })
        } else {
            None
        }
    }

    fn lookup(&self, bs: &mut Bitstream) -> u8 {
        let mut curr = self.root.as_ref();
        while let Some(node) = curr {
            match node {
                HuffNode8::Branch(b) => {
                    curr = Some(&b[bs.read_bit() as usize]);
                }
                HuffNode8::Leaf(v) => return *v,
            }
        }
        0
    }
}

enum HuffNode16 {
    Branch(Box<[HuffNode16; 2]>),
    Leaf(u16),
}

struct HuffTree16 {
    root: Option<HuffNode16>,
    low: Option<HuffTree8>,
    high: Option<HuffTree8>,
    escape_codes: [u16; 3],
    last_values: [u16; 3],
}

impl HuffTree16 {
    fn build(bs: &mut Bitstream, tree: &Self) -> Option<HuffNode16> {
        if bs.read_bit() != 0 {
            let left = Self::build(bs, tree)?;
            let right = Self::build(bs, tree)?;
            Some(HuffNode16::Branch(Box::new([left, right])))
        } else {
            let lo = tree.low.as_ref()?.lookup(bs);
            let hi = tree.high.as_ref()?.lookup(bs);
            Some(HuffNode16::Leaf(lo as u16 | ((hi as u16) << 8)))
        }
    }

    fn create(bs: &mut Bitstream) -> Option<Self> {
        if bs.read_bit() == 0 {
            return None;
        }
        let low = HuffTree8::create(bs);
        let high = HuffTree8::create(bs);
        
        let mut escape_codes = [0u16; 3];
        for i in 0..3 {
            escape_codes[i] = (bs.read_byte() as u16) | ((bs.read_byte() as u16) << 8);
        }
        
        let mut tree = Self {
            root: None,
            low,
            high,
            escape_codes,
            last_values: [0; 3],
        };
        
        tree.root = Self::build(bs, &tree);
        if bs.read_bit() != 0 {
            return None;
        }
        Some(tree)
    }

    fn reset_escape(&mut self) {
        self.last_values = [0; 3];
    }

    fn lookup(&mut self, bs: &mut Bitstream) -> u16 {
        let mut curr = self.root.as_ref();
        while let Some(node) = curr {
            match node {
                HuffNode16::Branch(b) => {
                    curr = Some(&b[bs.read_bit() as usize]);
                }
                HuffNode16::Leaf(v) => {
                    let mut val = *v;
                    for i in 0..3 {
                        if val == self.escape_codes[i] {
                            val = self.last_values[i];
                            break;
                        }
                    }
                    if val != self.last_values[0] {
                        self.last_values[2] = self.last_values[1];
                        self.last_values[1] = self.last_values[0];
                        self.last_values[0] = val;
                    }
                    return val;
                }
            }
        }
        0
    }
}

struct FrameData {
    palette: [ColorT; MAX_PALETTE],
    video: *mut u8,
    audio: [*mut u8; MAX_TRACKS],
    audio_len: [i32; MAX_TRACKS],
}

pub struct SmackerContext {
    fp: *mut libc::FILE,
    width: i32,
    height: i32,
    frames: i32,
    us_per_frame: i32,
    flags: i32,
    trees_size: i32,
    audio_size: [i32; 7],
    audio_rate: [i32; 7],
    frame_data_offset_in_file: c_long,
    frame_offsets: Vec<c_long>,
    frame_sizes: Vec<i32>,
    frame_types: Vec<u8>,
    mmap_tree: Option<HuffTree16>,
    mclr_tree: Option<HuffTree16>,
    full_tree: Option<HuffTree16>,
    type_tree: Option<HuffTree16>,
    frame_data: FrameData,
    current_frame: i32,
}

unsafe extern "C" {
    fn file_close(stream: *mut c_void) -> c_int;
}

const PALETTE_MAP: [u8; 64] = [
    0x00, 0x04, 0x08, 0x0C, 0x10, 0x14, 0x18, 0x1C,
    0x20, 0x24, 0x28, 0x2C, 0x30, 0x34, 0x38, 0x3C,
    0x41, 0x45, 0x49, 0x4D, 0x51, 0x55, 0x59, 0x5D,
    0x61, 0x65, 0x69, 0x6D, 0x71, 0x75, 0x79, 0x7D,
    0x82, 0x86, 0x8A, 0x8E, 0x92, 0x96, 0x9A, 0x9E,
    0xA2, 0xA6, 0xAA, 0xAE, 0xB2, 0xB6, 0xBA, 0xBE,
    0xC3, 0xC7, 0xCB, 0xCF, 0xD3, 0xD7, 0xDB, 0xDF,
    0xE3, 0xE7, 0xEB, 0xEF, 0xF3, 0xF7, 0xFB, 0xFF
];

const CHAIN_SIZE: [i32; 64] = [
     1,    2,    3,    4,    5,    6,    7,    8,
     9,   10,   11,   12,   13,   14,   15,   16,
    17,   18,   19,   20,   21,   22,   23,   24,
    25,   26,   27,   28,   29,   30,   31,   32,
    33,   34,   35,   36,   37,   38,   39,   40,
    41,   42,   43,   44,   45,   46,   47,   48,
    49,   50,   51,   52,   53,   54,   55,   56,
    57,   58,   59,  128,  256,  512, 1024, 2048
];

#[unsafe(no_mangle)]
pub unsafe extern "C" fn smacker_open(file: *mut libc::FILE) -> *mut SmackerContext {
    unsafe {
        if file.is_null() { return ptr::null_mut(); }
        
        let mut header = [0u8; HEADER_SIZE];
        if libc::fread(header.as_mut_ptr() as *mut c_void, 1, HEADER_SIZE, file) != HEADER_SIZE {
            return ptr::null_mut();
        }
        if &header[0..4] != b"SMK2" { return ptr::null_mut(); }
        
        let read_i32 = |off: usize| -> i32 {
            (header[off] as i32) | ((header[off+1] as i32) << 8) | ((header[off+2] as i32) << 16) | ((header[off+3] as i32) << 24)
        };
        
        let width = read_i32(4);
        let height = read_i32(8);
        let frames = read_i32(12);
        let frame_rate = read_i32(16);
        let us_per_frame = if frame_rate > 0 { frame_rate * 1000 } else if frame_rate < 0 { -10 * frame_rate } else { 100000 };
        let flags = read_i32(20);
        let mut audio_size = [0i32; 7];
        for i in 0..7 { audio_size[i] = read_i32(24 + 4*i); }
        let trees_size = read_i32(52);
        let mut audio_rate = [0i32; 7];
        for i in 0..7 { audio_rate[i] = read_i32(72 + 4*i); }
        
        let mut ctx = Box::new(SmackerContext {
            fp: file, width, height, frames, us_per_frame, flags, trees_size, audio_size, audio_rate,
            frame_data_offset_in_file: 0,
            frame_offsets: vec![0; frames as usize],
            frame_sizes: vec![0; frames as usize],
            frame_types: vec![0; frames as usize],
            mmap_tree: None, mclr_tree: None, full_tree: None, type_tree: None,
            frame_data: FrameData {
                palette: [0; MAX_PALETTE],
                video: libc::malloc((width * height) as usize) as *mut u8,
                audio: [ptr::null_mut(); MAX_TRACKS],
                audio_len: [0; MAX_TRACKS],
            },
            current_frame: -1,
        });
        
        for i in 0..MAX_TRACKS {
            if ctx.audio_rate[i] & AUDIO_FLAG_HAS_TRACK != 0 {
                ctx.frame_data.audio[i] = libc::malloc(ctx.audio_size[i] as usize) as *mut u8;
            }
        }
        
        // Read frame info
        let mut size_data = vec![0u8; 4 * frames as usize];
        libc::fread(size_data.as_mut_ptr() as *mut c_void, 1, size_data.len(), file);
        libc::fread(ctx.frame_types.as_mut_ptr() as *mut c_void, 1, frames as usize, file);
        
        let mut offset = 0;
        for i in 0..frames as usize {
            let s = ((size_data[4*i] as i32) | ((size_data[4*i+1] as i32) << 8) | ((size_data[4*i+2] as i32) << 16) | ((size_data[4*i+3] as i32) << 24)) & !3;
            ctx.frame_sizes[i] = s;
            ctx.frame_offsets[i] = offset;
            offset += s as c_long;
        }
        
        // Read trees
        let mut trees_data = vec![0u8; trees_size as usize];
        libc::fread(trees_data.as_mut_ptr() as *mut c_void, 1, trees_size as usize, file);
        let mut bs = Bitstream::new(&trees_data);
        ctx.mmap_tree = HuffTree16::create(&mut bs);
        ctx.mclr_tree = HuffTree16::create(&mut bs);
        ctx.full_tree = HuffTree16::create(&mut bs);
        ctx.type_tree = HuffTree16::create(&mut bs);
        
        ctx.frame_data_offset_in_file = libc::ftell(file);
        
        Box::into_raw(ctx)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn smacker_close(s: *mut SmackerContext) {
    unsafe {
        if s.is_null() { return; }
        let ctx = Box::from_raw(s);
        file_close(ctx.fp as *mut c_void);
        libc::free(ctx.frame_data.video as *mut c_void);
        for i in 0..MAX_TRACKS {
            libc::free(ctx.frame_data.audio[i] as *mut c_void);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn smacker_get_frames_info(s: *const SmackerContext, frame_count: *mut c_int, usf: *mut c_int) {
    unsafe {
        if s.is_null() { return; }
        if !frame_count.is_null() { *frame_count = (*s).frames; }
        if !usf.is_null() { *usf = (*s).us_per_frame; }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn smacker_get_video_info(s: *const SmackerContext, width: *mut c_int, height: *mut c_int, y_scale_mode: *mut c_int) {
    unsafe {
        if s.is_null() { return; }
        if !width.is_null() { *width = (*s).width; }
        if !height.is_null() { *height = (*s).height; }
        if !y_scale_mode.is_null() {
            *y_scale_mode = if (*s).flags & FLAG_Y_INTERLACE != 0 { 1 } else if (*s).flags & FLAG_Y_DOUBLE != 0 { 2 } else { 0 };
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn smacker_get_audio_info(s: *const SmackerContext, track: c_int, enabled: *mut c_int, channels: *mut c_int, bitdepth: *mut c_int, audio_rate: *mut c_int) {
    unsafe {
        if s.is_null() || track < 0 || track >= 7 { return; }
        let rate = (*s).audio_rate[track as usize];
        let has_track = (rate & AUDIO_FLAG_HAS_TRACK) != 0;
        if !enabled.is_null() { *enabled = has_track as c_int; }
        if !channels.is_null() { *channels = if has_track { if (rate & AUDIO_FLAG_STEREO) != 0 { 2 } else { 1 } } else { 0 }; }
        if !bitdepth.is_null() { *bitdepth = if has_track { if (rate & AUDIO_FLAG_16BIT) != 0 { 16 } else { 8 } } else { 0 }; }
        if !audio_rate.is_null() { *audio_rate = rate & 0xFFFFFF; }
    }
}

unsafe fn decode_palette(ctx: &mut SmackerContext, data: &[u8]) {
    let mut new_palette = ctx.frame_data.palette;
    let mut idx = 0;
    let mut color_idx = 0;
    while idx < data.len() && color_idx < MAX_PALETTE {
        if data[idx] & 0x80 != 0 {
            let num = 1 + (data[idx] & 0x7f) as usize;
            idx += 1;
            for i in 0..num {
                if color_idx + i < MAX_PALETTE {
                    new_palette[color_idx + i] = ctx.frame_data.palette[color_idx + i];
                }
            }
            color_idx += num;
        } else if data[idx] & 0x40 != 0 {
            let num = 1 + (data[idx] & 0x3f) as usize;
            if idx + 1 >= data.len() { break; }
            let offset = data[idx+1] as usize;
            idx += 2;
            for i in 0..num {
                if color_idx + i < MAX_PALETTE && offset + i < MAX_PALETTE {
                    new_palette[color_idx + i] = ctx.frame_data.palette[offset + i];
                }
            }
            color_idx += num;
        } else {
            if idx + 2 >= data.len() { break; }
            new_palette[color_idx] = ((PALETTE_MAP[(data[idx] & 0x3f) as usize] as u32) << 16) |
                                     ((PALETTE_MAP[(data[idx+1] & 0x3f) as usize] as u32) << 8) |
                                     (PALETTE_MAP[(data[idx+2] & 0x3f) as usize] as u32);
            color_idx += 1;
            idx += 3;
        }
    }
    ctx.frame_data.palette = new_palette;
}

unsafe fn decode_video(ctx: &mut SmackerContext, data: &[u8]) {
    unsafe {
        if let Some(ref mut t) = ctx.mclr_tree { t.reset_escape(); }
        if let Some(ref mut t) = ctx.mmap_tree { t.reset_escape(); }
        if let Some(ref mut t) = ctx.full_tree { t.reset_escape(); }
        if let Some(ref mut t) = ctx.type_tree { t.reset_escape(); }
        
        let mut bs = Bitstream::new(data);
        let mut chain = 0;
        let mut block_type = 0;
        let mut solid_color = 0u8;
        
        for row in (0..ctx.height).step_by(4) {
            for col in (0..ctx.width).step_by(4) {
                if chain <= 0 {
                    if let Some(ref mut t) = ctx.type_tree {
                        let val = t.lookup(&mut bs);
                        block_type = val & 3;
                        chain = CHAIN_SIZE[((val >> 2) & 0x3f) as usize];
                        solid_color = (val >> 8) as u8;
                    }
                }
                match block_type as i32 {
                    BLOCK_MONO => {
                        let colors = ctx.mclr_tree.as_mut().map(|t| t.lookup(&mut bs)).unwrap_or(0);
                        let c1 = (colors & 0xff) as u8;
                        let c2 = (colors >> 8) as u8;
                        let mut map = ctx.mmap_tree.as_mut().map(|t| t.lookup(&mut bs)).unwrap_or(0);
                        for y in 0..4 {
                            let p = ctx.frame_data.video.add(((row + y) * ctx.width + col) as usize);
                            for x in 0..4 {
                                *p.add(x) = if (map & (1 << x)) != 0 { c2 } else { c1 };
                            }
                            map >>= 4;
                        }
                    }
                    BLOCK_FULL => {
                        for y in 0..4 {
                            let p = ctx.frame_data.video.add(((row + y) * ctx.width + col) as usize);
                            let colors1 = ctx.full_tree.as_mut().map(|t| t.lookup(&mut bs)).unwrap_or(0);
                            *p.add(2) = (colors1 & 0xff) as u8;
                            *p.add(3) = (colors1 >> 8) as u8;
                            let colors2 = ctx.full_tree.as_mut().map(|t| t.lookup(&mut bs)).unwrap_or(0);
                            *p.add(0) = (colors2 & 0xff) as u8;
                            *p.add(1) = (colors2 >> 8) as u8;
                        }
                    }
                    BLOCK_SOLID => {
                        for y in 0..4 {
                            let p = ctx.frame_data.video.add(((row + y) * ctx.width + col) as usize);
                            for x in 0..4 { *p.add(x) = solid_color; }
                        }
                    }
                    _ => {}
                }
                chain -= 1;
            }
        }
    }
}

unsafe fn decode_audio(ctx: &mut SmackerContext, track: usize, data: &[u8]) {
    unsafe {
        if (ctx.audio_rate[track] & AUDIO_FLAG_COMPRESSED) == 0 {
            ptr::copy_nonoverlapping(data.as_ptr(), ctx.frame_data.audio[track], data.len());
            ctx.frame_data.audio_len[track] = data.len() as i32;
        }
    }
}

unsafe fn decode_frame(ctx: &mut SmackerContext) -> SmackerFrameStatus {
    unsafe {
        if ctx.current_frame >= ctx.frames { return SmackerFrameStatus::Done; }
        
        let size = ctx.frame_sizes[ctx.current_frame as usize] as usize;
        let mut data = vec![0u8; size];
        libc::fseek(ctx.fp, ctx.frame_data_offset_in_file + ctx.frame_offsets[ctx.current_frame as usize], libc::SEEK_SET);
        libc::fread(data.as_mut_ptr() as *mut c_void, 1, size, ctx.fp);
        
        let ftype = ctx.frame_types[ctx.current_frame as usize];
        let mut data_idx = 0;
        if ftype & 1 != 0 {
            let pal_size = data[0] as usize * 4;
            decode_palette(ctx, &data[1..pal_size]);
            data_idx += pal_size;
        }
        for i in 0..MAX_TRACKS {
            if ftype & (1 << (i + 1)) != 0 {
                let len = ((data[data_idx] as u32) | ((data[data_idx+1] as u32) << 8) | ((data[data_idx+2] as u32) << 16) | ((data[data_idx+3] as u32) << 24)) as usize;
                decode_audio(ctx, i, &data[data_idx+4 .. data_idx+len]);
                data_idx += len;
            } else {
                ctx.frame_data.audio_len[i] = 0;
            }
        }
        decode_video(ctx, &data[data_idx..]);
        SmackerFrameStatus::Ok
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn smacker_first_frame(s: *mut SmackerContext) -> SmackerFrameStatus {
    unsafe {
        if s.is_null() { return SmackerFrameStatus::Error; }
        (*s).current_frame = 0;
        decode_frame(&mut *s)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn smacker_next_frame(s: *mut SmackerContext) -> SmackerFrameStatus {
    unsafe {
        if s.is_null() { return SmackerFrameStatus::Error; }
        (*s).current_frame += 1;
        decode_frame(&mut *s)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn smacker_get_frame_palette(s: *const SmackerContext) -> *const ColorT {
    unsafe { if s.is_null() { ptr::null() } else { (*s).frame_data.palette.as_ptr() } }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn smacker_get_frame_video(s: *const SmackerContext) -> *const u8 {
    unsafe { if s.is_null() { ptr::null() } else { (*s).frame_data.video } }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn smacker_get_frame_audio_size(s: *const SmackerContext, track: c_int) -> c_int {
    unsafe { if s.is_null() || track < 0 || track >= 7 { 0 } else { (*s).frame_data.audio_len[track as usize] } }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn smacker_get_frame_audio(s: *const SmackerContext, track: c_int) -> *const u8 {
    unsafe { if s.is_null() || track < 0 || track >= 7 { ptr::null() } else { (*s).frame_data.audio[track as usize] } }
}
