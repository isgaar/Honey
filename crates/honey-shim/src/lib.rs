//! C ABI drop-in replacement library for low-level OS FreeType substitution.
//!
//! Exposes standard FreeType 2 structs and functions so that GTK, Qt, Cairo,
//! HarfBuzz, and Chromium/Skia can link or preload directly against Honey's Quartz engine.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use honey_core::{
    compute_dilation_factor_with_profile, dilate_mask, DesktopProfile, GlyphOutline, HoneyFace,
    OutlineBuilder, QuartzRasterizer,
};
use std::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ulong, c_ushort, c_void, CStr, CString};
use std::fs;
use std::ptr;
use std::slice;
use std::sync::OnceLock;

fn get_active_profile() -> DesktopProfile {
    static PROFILE: OnceLock<DesktopProfile> = OnceLock::new();
    *PROFILE.get_or_init(|| {
        if let Ok(p) = std::env::var("HONEY_PROFILE") {
            match p.to_lowercase().as_str() {
                "gnome" => DesktopProfile::Gnome,
                "kde" => DesktopProfile::Kde,
                _ => DesktopProfile::Standard,
            }
        } else if let Ok(d) = std::env::var("XDG_CURRENT_DESKTOP") {
            let upper = d.to_uppercase();
            if upper.contains("GNOME") {
                DesktopProfile::Gnome
            } else if upper.contains("KDE") {
                DesktopProfile::Kde
            } else {
                DesktopProfile::Standard
            }
        } else {
            DesktopProfile::Standard
        }
    })
}

extern "C" {
    fn calloc(nmemb: usize, size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

pub const FT_GLYPH_FORMAT_BITMAP: c_uint = 0x6269_7473; // 'bits'
pub const FT_GLYPH_FORMAT_OUTLINE: c_uint = 0x6f75_746c; // 'outl'
pub const FT_PIXEL_MODE_NONE: c_uchar = 0;
pub const FT_PIXEL_MODE_MONO: c_uchar = 1;
pub const FT_PIXEL_MODE_GRAY: c_uchar = 2;
pub const FT_PIXEL_MODE_GRAY2: c_uchar = 3;
pub const FT_PIXEL_MODE_GRAY4: c_uchar = 4;
pub const FT_PIXEL_MODE_LCD: c_uchar = 5;
pub const FT_PIXEL_MODE_LCD_V: c_uchar = 6;
pub const FT_PIXEL_MODE_BGRA: c_uchar = 7;

pub const FT_LOAD_DEFAULT: c_int = 0;
pub const FT_LOAD_NO_SCALE: c_int = 1 << 0;
pub const FT_LOAD_NO_HINTING: c_int = 1 << 1;
pub const FT_LOAD_RENDER: c_int = 1 << 2;
pub const FT_LOAD_NO_BITMAP: c_int = 1 << 3;

pub const FT_KERNING_DEFAULT: c_uint = 0;
pub const FT_KERNING_UNFITTED: c_uint = 1;
pub const FT_KERNING_UNSCALED: c_uint = 2;

pub const FT_FACE_FLAG_SCALABLE: c_long = 1 << 0;
pub const FT_FACE_FLAG_FIXED_SIZES: c_long = 1 << 1;
pub const FT_FACE_FLAG_FIXED_WIDTH: c_long = 1 << 2;
pub const FT_FACE_FLAG_SFNT: c_long = 1 << 3;
pub const FT_FACE_FLAG_HORIZONTAL: c_long = 1 << 4;
pub const FT_FACE_FLAG_VERTICAL: c_long = 1 << 5;
pub const FT_FACE_FLAG_KERNING: c_long = 1 << 6;
pub const FT_FACE_FLAG_FAST_GLYPHS: c_long = 1 << 7;
pub const FT_FACE_FLAG_MULTIPLE_MASTERS: c_long = 1 << 8;
pub const FT_FACE_FLAG_GLYPH_NAMES: c_long = 1 << 9;
pub const FT_FACE_FLAG_EXTERNAL_STREAM: c_long = 1 << 10;
pub const FT_FACE_FLAG_HINTER: c_long = 1 << 11;
pub const FT_FACE_FLAG_CID_KEYED: c_long = 1 << 12;
pub const FT_FACE_FLAG_TRICKY: c_long = 1 << 13;
pub const FT_FACE_FLAG_COLOR: c_long = 1 << 14;
pub const FT_FACE_FLAG_VARIATION: c_long = 1 << 15;

pub const FT_STYLE_FLAG_ITALIC: c_long = 1 << 0;
pub const FT_STYLE_FLAG_BOLD: c_long = 1 << 1;

pub const FT_SFNT_HEAD: c_uint = 0;
pub const FT_SFNT_MAXP: c_uint = 1;
pub const FT_SFNT_OS2: c_uint = 2;
pub const FT_SFNT_HHEA: c_uint = 3;
pub const FT_SFNT_VHEA: c_uint = 4;
pub const FT_SFNT_POST: c_uint = 5;
pub const FT_SFNT_PCLT: c_uint = 6;

#[repr(C)]
pub struct FT_Bitmap {
    pub rows: c_uint,
    pub width: c_uint,
    pub pitch: c_int,
    pub buffer: *mut c_uchar,
    pub num_grays: c_ushort,
    pub pixel_mode: c_uchar,
    pub palette_mode: c_uchar,
    pub palette: *mut c_void,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct FT_Glyph_Metrics {
    pub width: c_long,
    pub height: c_long,
    pub horiBearingX: c_long,
    pub horiBearingY: c_long,
    pub horiAdvance: c_long,
    pub vertBearingX: c_long,
    pub vertBearingY: c_long,
    pub vertAdvance: c_long,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct FT_Vector {
    pub x: c_long,
    pub y: c_long,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct FT_BBox {
    pub xMin: c_long,
    pub yMin: c_long,
    pub xMax: c_long,
    pub yMax: c_long,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct FT_Matrix {
    pub xx: c_long,
    pub xy: c_long,
    pub yx: c_long,
    pub yy: c_long,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FT_Outline {
    pub n_contours: c_short,
    pub n_points: c_short,
    pub points: *mut FT_Vector,
    pub tags: *mut c_char,
    pub contours: *mut c_short,
    pub flags: c_int,
}

pub type FT_Outline_MoveToFunc =
    Option<unsafe extern "C" fn(to: *const FT_Vector, user: *mut c_void) -> c_int>;
pub type FT_Outline_LineToFunc =
    Option<unsafe extern "C" fn(to: *const FT_Vector, user: *mut c_void) -> c_int>;
pub type FT_Outline_ConicToFunc = Option<
    unsafe extern "C" fn(control: *const FT_Vector, to: *const FT_Vector, user: *mut c_void) -> c_int,
>;
pub type FT_Outline_CubicToFunc = Option<
    unsafe extern "C" fn(
        control1: *const FT_Vector,
        control2: *const FT_Vector,
        to: *const FT_Vector,
        user: *mut c_void,
    ) -> c_int,
>;

#[repr(C)]
pub struct FT_Outline_Funcs {
    pub move_to: FT_Outline_MoveToFunc,
    pub line_to: FT_Outline_LineToFunc,
    pub conic_to: FT_Outline_ConicToFunc,
    pub cubic_to: FT_Outline_CubicToFunc,
    pub shift: c_int,
    pub delta: c_long,
}

#[repr(C)]
pub struct FT_GlyphSlotRec {
    pub library: *mut c_void,
    pub face: *mut FT_FaceRec,
    pub next: *mut FT_GlyphSlotRec,
    pub glyph_index: c_uint,
    pub generic: [usize; 2],
    pub metrics: FT_Glyph_Metrics,
    pub linearHoriAdvance: c_long,
    pub linearVertAdvance: c_long,
    pub advance: FT_Vector,
    pub format: c_uint,
    pub bitmap: FT_Bitmap,
    pub bitmap_left: c_int,
    pub bitmap_top: c_int,
    pub outline: FT_Outline,
    pub num_subglyphs: c_uint,
    pub subglyphs: *mut c_void,
    pub control_data: *mut c_void,
    pub control_len: c_long,
    pub lsb_delta: c_long,
    pub rsb_delta: c_long,
    pub other: *mut c_void,
    pub internal: *mut c_void,
}

#[repr(C)]
pub struct FT_CharMapRec {
    pub face: *mut FT_FaceRec,
    pub encoding: c_uint,
    pub platform_id: c_ushort,
    pub encoding_id: c_ushort,
}

#[repr(C)]
pub struct FT_Size_Metrics {
    pub x_ppem: c_ushort,
    pub y_ppem: c_ushort,
    pub x_scale: c_long,
    pub y_scale: c_long,
    pub ascender: c_long,
    pub descender: c_long,
    pub height: c_long,
    pub max_advance: c_long,
}

#[repr(C)]
pub struct FT_SizeRec {
    pub face: *mut FT_FaceRec,
    pub generic: [usize; 2],
    pub metrics: FT_Size_Metrics,
    pub internal: *mut c_void,
}

#[repr(C)]
pub struct FT_FaceRec {
    pub num_faces: c_long,
    pub face_index: c_long,
    pub face_flags: c_long,
    pub style_flags: c_long,
    pub num_glyphs: c_long,
    pub family_name: *mut c_char,
    pub style_name: *mut c_char,
    pub num_fixed_sizes: c_int,
    pub available_sizes: *mut c_void,
    pub num_charmaps: c_int,
    pub charmaps: *mut *mut FT_CharMapRec,
    pub generic: [usize; 2],
    pub bbox: FT_BBox,
    pub units_per_EM: c_ushort,
    pub ascender: c_short,
    pub descender: c_short,
    pub height: c_short,
    pub max_advance_width: c_short,
    pub max_advance_height: c_short,
    pub underline_position: c_short,
    pub underline_thickness: c_short,
    pub glyph: *mut FT_GlyphSlotRec,
    pub size: *mut FT_SizeRec,
    pub charmap: *mut FT_CharMapRec,

    pub driver: *mut c_void,
    pub memory: *mut c_void,
    pub stream: *mut c_void,
    pub sizes_list: [usize; 2],
    pub autohint: [usize; 2],
    pub extensions: *mut c_void,
    pub internal: *mut FaceContext,
}

#[repr(C)]
pub struct FT_GlyphRec {
    pub library: *mut c_void,
    pub clazz: *const c_void,
    pub format: c_uint,
    pub advance: FT_Vector,
}

#[repr(C)]
pub struct FT_BitmapGlyphRec {
    pub root: FT_GlyphRec,
    pub left: c_int,
    pub top: c_int,
    pub bitmap: FT_Bitmap,
}

#[repr(C)]
pub struct FT_OutlineGlyphRec {
    pub root: FT_GlyphRec,
    pub outline: FT_Outline,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SfntNameRecord {
    pub platform_id: c_ushort,
    pub encoding_id: c_ushort,
    pub language_id: c_ushort,
    pub name_id: c_ushort,
    pub offset: usize,
    pub len: usize,
}

#[repr(C)]
pub struct TT_OS2 {
    pub version: c_ushort,
    pub xAvgCharWidth: c_short,
    pub usWeightClass: c_ushort,
    pub usWidthClass: c_ushort,
    pub fsType: c_ushort,
    pub ySubscriptXSize: c_short,
    pub ySubscriptYSize: c_short,
    pub ySubscriptXOffset: c_short,
    pub ySubscriptYOffset: c_short,
    pub ySuperscriptXSize: c_short,
    pub ySuperscriptYSize: c_short,
    pub ySuperscriptXOffset: c_short,
    pub ySuperscriptYOffset: c_short,
    pub yStrikeoutSize: c_short,
    pub yStrikeoutPosition: c_short,
    pub sFamilyClass: c_short,
    pub panose: [c_uchar; 10],
    pub ulUnicodeRange1: c_ulong,
    pub ulUnicodeRange2: c_ulong,
    pub ulUnicodeRange3: c_ulong,
    pub ulUnicodeRange4: c_ulong,
    pub achVendID: [c_char; 4],
    pub fsSelection: c_ushort,
    pub usFirstCharIndex: c_ushort,
    pub usLastCharIndex: c_ushort,
    pub sTypoAscender: c_short,
    pub sTypoDescender: c_short,
    pub sTypoLineGap: c_short,
    pub usWinAscent: c_ushort,
    pub usWinDescent: c_ushort,
    pub ulCodePageRange1: c_ulong,
    pub ulCodePageRange2: c_ulong,
    pub sxHeight: c_short,
    pub sCapHeight: c_short,
    pub usDefaultChar: c_ushort,
    pub usBreakChar: c_ushort,
    pub usMaxContext: c_ushort,
    pub usLowerOpticalPointSize: c_ushort,
    pub usUpperOpticalPointSize: c_ushort,
}

fn parse_os2_table(data: &[u8]) -> Option<TT_OS2> {
    if data.len() < 78 {
        return None;
    }
    let read_u16 = |off: usize| -> u16 {
        if off + 2 <= data.len() {
            u16::from_be_bytes([data[off], data[off + 1]])
        } else {
            0
        }
    };
    let read_i16 = |off: usize| -> i16 { read_u16(off) as i16 };
    let read_u32 = |off: usize| -> u32 {
        if off + 4 <= data.len() {
            u32::from_be_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]])
        } else {
            0
        }
    };

    let version = read_u16(0);
    let mut panose = [0u8; 10];
    if data.len() >= 42 {
        panose.copy_from_slice(&data[32..42]);
    }
    let mut vend = [0 as c_char; 4];
    if data.len() >= 62 {
        for i in 0..4 {
            vend[i] = data[58 + i] as c_char;
        }
    }

    Some(TT_OS2 {
        version,
        xAvgCharWidth: read_i16(2),
        usWeightClass: read_u16(4),
        usWidthClass: read_u16(6),
        fsType: read_u16(8),
        ySubscriptXSize: read_i16(10),
        ySubscriptYSize: read_i16(12),
        ySubscriptXOffset: read_i16(14),
        ySubscriptYOffset: read_i16(16),
        ySuperscriptXSize: read_i16(18),
        ySuperscriptYSize: read_i16(20),
        ySuperscriptXOffset: read_i16(22),
        ySuperscriptYOffset: read_i16(24),
        yStrikeoutSize: read_i16(26),
        yStrikeoutPosition: read_i16(28),
        sFamilyClass: read_i16(30),
        panose,
        ulUnicodeRange1: read_u32(42) as c_ulong,
        ulUnicodeRange2: read_u32(46) as c_ulong,
        ulUnicodeRange3: read_u32(50) as c_ulong,
        ulUnicodeRange4: read_u32(54) as c_ulong,
        achVendID: vend,
        fsSelection: read_u16(62),
        usFirstCharIndex: read_u16(64),
        usLastCharIndex: read_u16(66),
        sTypoAscender: read_i16(68),
        sTypoDescender: read_i16(70),
        sTypoLineGap: read_i16(72),
        usWinAscent: read_u16(74),
        usWinDescent: read_u16(76),
        ulCodePageRange1: if version >= 1 { read_u32(78) as c_ulong } else { 0 },
        ulCodePageRange2: if version >= 1 { read_u32(82) as c_ulong } else { 0 },
        sxHeight: if version >= 2 { read_i16(86) } else { 0 },
        sCapHeight: if version >= 2 { read_i16(88) } else { 0 },
        usDefaultChar: if version >= 2 { read_u16(90) } else { 0 },
        usBreakChar: if version >= 2 { read_u16(92) } else { 0 },
        usMaxContext: if version >= 2 { read_u16(94) } else { 0 },
        usLowerOpticalPointSize: if version >= 5 { read_u16(96) } else { 0 },
        usUpperOpticalPointSize: if version >= 5 { read_u16(98) } else { 0 },
    })
}

#[repr(C)]
pub struct TT_Header {
    pub Table_Version: c_long,
    pub Font_Revision: c_long,
    pub CheckSum_Adjust: c_long,
    pub Magic_Number: c_long,
    pub Flags: c_ushort,
    pub Units_Per_EM: c_ushort,
    pub Created: [c_ulong; 2],
    pub Modified: [c_ulong; 2],
    pub xMin: c_short,
    pub yMin: c_short,
    pub xMax: c_short,
    pub yMax: c_short,
    pub Mac_Style: c_ushort,
    pub Lowest_Rec_PPEM: c_ushort,
    pub Font_Direction: c_short,
    pub Index_To_Loc_Format: c_short,
    pub Glyph_Data_Format: c_short,
}

#[repr(C)]
pub struct TT_HoriHeader {
    pub Version: c_long,
    pub Ascender: c_short,
    pub Descender: c_short,
    pub Line_Gap: c_short,
    pub advance_Width_Max: c_ushort,
    pub min_Left_Side_Bearing: c_short,
    pub min_Right_Side_Bearing: c_short,
    pub xMax_Extent: c_short,
    pub caret_Slope_Rise: c_short,
    pub caret_Slope_Run: c_short,
    pub caret_Offset: c_short,
    pub Reserved: [c_short; 4],
    pub metric_Data_Format: c_short,
    pub number_Of_HMetrics: c_ushort,
    pub long_metrics: *mut c_void,
    pub short_metrics: *mut c_void,
}

#[repr(C)]
pub struct TT_Postscript {
    pub FormatType: c_long,
    pub italicAngle: c_long,
    pub underlinePosition: c_short,
    pub underlineThickness: c_short,
    pub isFixedPitch: c_ulong,
    pub minMemType42: c_ulong,
    pub maxMemType42: c_ulong,
    pub minMemType1: c_ulong,
    pub maxMemType1: c_ulong,
}

#[repr(C)]
pub struct TT_MaxProfile {
    pub version: c_long,
    pub numGlyphs: c_ushort,
    pub maxPoints: c_ushort,
    pub maxContours: c_ushort,
    pub maxCompositePoints: c_ushort,
    pub maxCompositeContours: c_ushort,
    pub maxZones: c_ushort,
    pub maxTwilightPoints: c_ushort,
    pub maxStorage: c_ushort,
    pub maxFunctionDefs: c_ushort,
    pub maxInstructionDefs: c_ushort,
    pub maxStackElements: c_ushort,
    pub maxSizeOfInstructions: c_ushort,
    pub maxComponentElements: c_ushort,
    pub maxComponentDepth: c_ushort,
}

fn parse_head_table(data: &[u8]) -> Option<TT_Header> {
    if data.len() < 54 {
        return None;
    }
    let read_u16 = |off: usize| -> u16 {
        u16::from_be_bytes([data[off], data[off + 1]])
    };
    let read_i16 = |off: usize| -> i16 { read_u16(off) as i16 };
    let read_u32 = |off: usize| -> u32 {
        u32::from_be_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]])
    };
    let read_i32 = |off: usize| -> i32 { read_u32(off) as i32 };

    Some(TT_Header {
        Table_Version: read_i32(0) as c_long,
        Font_Revision: read_i32(4) as c_long,
        CheckSum_Adjust: read_i32(8) as c_long,
        Magic_Number: read_u32(12) as c_long,
        Flags: read_u16(16),
        Units_Per_EM: read_u16(18),
        Created: [read_u32(20) as c_ulong, read_u32(24) as c_ulong],
        Modified: [read_u32(28) as c_ulong, read_u32(32) as c_ulong],
        xMin: read_i16(36),
        yMin: read_i16(38),
        xMax: read_i16(40),
        yMax: read_i16(42),
        Mac_Style: read_u16(44),
        Lowest_Rec_PPEM: read_u16(46),
        Font_Direction: read_i16(48),
        Index_To_Loc_Format: read_i16(50),
        Glyph_Data_Format: read_i16(52),
    })
}

fn parse_hhea_table(data: &[u8]) -> Option<TT_HoriHeader> {
    if data.len() < 36 {
        return None;
    }
    let read_u16 = |off: usize| -> u16 {
        u16::from_be_bytes([data[off], data[off + 1]])
    };
    let read_i16 = |off: usize| -> i16 { read_u16(off) as i16 };
    let read_i32 = |off: usize| -> i32 {
        i32::from_be_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]])
    };

    Some(TT_HoriHeader {
        Version: read_i32(0) as c_long,
        Ascender: read_i16(4),
        Descender: read_i16(6),
        Line_Gap: read_i16(8),
        advance_Width_Max: read_u16(10),
        min_Left_Side_Bearing: read_i16(12),
        min_Right_Side_Bearing: read_i16(14),
        xMax_Extent: read_i16(16),
        caret_Slope_Rise: read_i16(18),
        caret_Slope_Run: read_i16(20),
        caret_Offset: read_i16(22),
        Reserved: [read_i16(24), read_i16(26), read_i16(28), read_i16(30)],
        metric_Data_Format: read_i16(32),
        number_Of_HMetrics: read_u16(34),
        long_metrics: ptr::null_mut(),
        short_metrics: ptr::null_mut(),
    })
}

fn parse_post_table(data: &[u8]) -> Option<TT_Postscript> {
    if data.len() < 32 {
        return None;
    }
    let read_u16 = |off: usize| -> u16 {
        u16::from_be_bytes([data[off], data[off + 1]])
    };
    let read_i16 = |off: usize| -> i16 { read_u16(off) as i16 };
    let read_u32 = |off: usize| -> u32 {
        u32::from_be_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]])
    };
    let read_i32 = |off: usize| -> i32 { read_u32(off) as i32 };

    Some(TT_Postscript {
        FormatType: read_i32(0) as c_long,
        italicAngle: read_i32(4) as c_long,
        underlinePosition: read_i16(8),
        underlineThickness: read_i16(10),
        isFixedPitch: read_u32(12) as c_ulong,
        minMemType42: read_u32(16) as c_ulong,
        maxMemType42: read_u32(20) as c_ulong,
        minMemType1: read_u32(24) as c_ulong,
        maxMemType1: read_u32(28) as c_ulong,
    })
}

fn parse_maxp_table(data: &[u8]) -> Option<TT_MaxProfile> {
    if data.len() < 6 {
        return None;
    }
    let read_u16 = |off: usize| -> u16 {
        u16::from_be_bytes([data[off], data[off + 1]])
    };
    let read_u32 = |off: usize| -> u32 {
        u32::from_be_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]])
    };

    let ver = read_u32(0);
    let num_glyphs = read_u16(4);

    let (max_points, max_contours, max_comp_pts, max_comp_contours, max_zones, max_twilight, max_storage, max_func_defs, max_instr_defs, max_stack, max_size_instr, max_comp_elem, max_comp_depth) =
        if data.len() >= 32 {
            (
                read_u16(6),
                read_u16(8),
                read_u16(10),
                read_u16(12),
                read_u16(14),
                read_u16(16),
                read_u16(18),
                read_u16(20),
                read_u16(22),
                read_u16(24),
                read_u16(26),
                read_u16(28),
                read_u16(30),
            )
        } else {
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)
        };

    Some(TT_MaxProfile {
        version: ver as c_long,
        numGlyphs: num_glyphs,
        maxPoints: max_points,
        maxContours: max_contours,
        maxCompositePoints: max_comp_pts,
        maxCompositeContours: max_comp_contours,
        maxZones: max_zones,
        maxTwilightPoints: max_twilight,
        maxStorage: max_storage,
        maxFunctionDefs: max_func_defs,
        maxInstructionDefs: max_instr_defs,
        maxStackElements: max_stack,
        maxSizeOfInstructions: max_size_instr,
        maxComponentElements: max_comp_elem,
        maxComponentDepth: max_comp_depth,
    })
}

/// Internal state holding font bytes, metrics, and rasterization properties.
pub struct FaceContext {
    pub font_data: Vec<u8>,
    pub actual_face_index: u32,
    pub pixel_size: f32,
    pub last_bitmap_buffer: Vec<u8>,
    pub outline_points: Vec<FT_Vector>,
    pub outline_tags: Vec<c_char>,
    pub outline_contours: Vec<c_short>,
    pub charmap_obj: FT_CharMapRec,
    pub charmaps_array: Vec<*mut FT_CharMapRec>,
    pub size_obj: FT_SizeRec,
    pub transform_matrix: Option<FT_Matrix>,
    pub transform_delta: Option<FT_Vector>,
    pub sfnt_names: Vec<SfntNameRecord>,
    pub os2_table: Option<Box<TT_OS2>>,
    pub head_table: Option<Box<TT_Header>>,
    pub hhea_table: Option<Box<TT_HoriHeader>>,
    pub post_table: Option<Box<TT_Postscript>>,
    pub maxp_table: Option<Box<TT_MaxProfile>>,
}

struct OutlineAccumulator {
    points: Vec<FT_Vector>,
    tags: Vec<c_char>,
    contours: Vec<c_short>,
    scale: f32,
}

impl OutlineAccumulator {
    fn new(scale: f32) -> Self {
        Self {
            points: Vec::new(),
            tags: Vec::new(),
            contours: Vec::new(),
            scale,
        }
    }
}

impl OutlineBuilder for OutlineAccumulator {
    fn move_to(&mut self, x: f32, y: f32) {
        if !self.points.is_empty() {
            let last_idx = (self.points.len() - 1) as c_short;
            if self.contours.last() != Some(&last_idx) {
                self.contours.push(last_idx);
            }
        }
        self.points.push(FT_Vector {
            x: (x * self.scale * 64.0) as c_long,
            y: (y * self.scale * 64.0) as c_long,
        });
        self.tags.push(1); // On-curve
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.points.push(FT_Vector {
            x: (x * self.scale * 64.0) as c_long,
            y: (y * self.scale * 64.0) as c_long,
        });
        self.tags.push(1); // On-curve
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.points.push(FT_Vector {
            x: (x1 * self.scale * 64.0) as c_long,
            y: (y1 * self.scale * 64.0) as c_long,
        });
        self.tags.push(0); // Off-curve conic

        self.points.push(FT_Vector {
            x: (x * self.scale * 64.0) as c_long,
            y: (y * self.scale * 64.0) as c_long,
        });
        self.tags.push(1); // On-curve
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.points.push(FT_Vector {
            x: (x1 * self.scale * 64.0) as c_long,
            y: (y1 * self.scale * 64.0) as c_long,
        });
        self.tags.push(2); // Off-curve cubic

        self.points.push(FT_Vector {
            x: (x2 * self.scale * 64.0) as c_long,
            y: (y2 * self.scale * 64.0) as c_long,
        });
        self.tags.push(2); // Off-curve cubic

        self.points.push(FT_Vector {
            x: (x * self.scale * 64.0) as c_long,
            y: (y * self.scale * 64.0) as c_long,
        });
        self.tags.push(1); // On-curve
    }

    fn close(&mut self) {
        if !self.points.is_empty() {
            let last_idx = (self.points.len() - 1) as c_short;
            if self.contours.last() != Some(&last_idx) {
                self.contours.push(last_idx);
            }
        }
    }
}

// =========================================================================
// Honey Custom Diagnostic & Telemetry API
// =========================================================================

#[no_mangle]
pub extern "C" fn honey_version() -> *const c_char {
    static VERSION: &[u8] = b"Honey Core 0.7.0 (Rust Quartz Parity Engine)\0";
    VERSION.as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn honey_blend_pixel_linear(bg_argb: u32, fg_argb: u32, alpha: u8) -> u32 {
    honey_core::blend_linear(bg_argb, fg_argb, alpha)
}

#[no_mangle]
pub extern "C" fn honey_dilate_mask(
    src: *const u8,
    dst: *mut u8,
    width: usize,
    height: usize,
    factor: f32,
) -> c_int {
    if src.is_null() || dst.is_null() || width == 0 || height == 0 {
        return -1;
    }
    let src_slice = unsafe { slice::from_raw_parts(src, width * height) };
    let dilated = honey_core::dilate_mask(src_slice, width, height, factor);
    unsafe {
        ptr::copy_nonoverlapping(dilated.as_ptr(), dst, width * height);
    }
    0
}

#[no_mangle]
pub extern "C" fn honey_compute_ink_energy(mask: *const u8, len: usize) -> u64 {
    if mask.is_null() || len == 0 {
        return 0;
    }
    let mask_slice = unsafe { slice::from_raw_parts(mask, len) };
    honey_core::compute_ink_energy(mask_slice)
}

// =========================================================================
// FreeType 2 C ABI Implementation
// =========================================================================

#[no_mangle]
pub extern "C" fn FT_Init_FreeType(alibrary: *mut *mut c_void) -> c_int {
    if alibrary.is_null() {
        return 1;
    }
    unsafe {
        *alibrary = 0xCAFE_BABE as *mut c_void;
    }
    0
}

#[no_mangle]
pub extern "C" fn FT_Done_FreeType(library: *mut c_void) -> c_int {
    let _ = library;
    0
}

#[no_mangle]
pub extern "C" fn FT_New_Library(_memory: *mut c_void, alibrary: *mut *mut c_void) -> c_int {
    FT_Init_FreeType(alibrary)
}

#[no_mangle]
pub extern "C" fn FT_Done_Library(library: *mut c_void) -> c_int {
    FT_Done_FreeType(library)
}

#[no_mangle]
pub extern "C" fn FT_Reference_Library(library: *mut c_void) -> c_int {
    let _ = library;
    0
}

#[no_mangle]
pub extern "C" fn FT_Add_Default_Modules(library: *mut c_void) {
    let _ = library;
}

#[no_mangle]
pub extern "C" fn FT_Set_Default_Properties(library: *mut c_void) {
    let _ = library;
}

#[no_mangle]
pub extern "C" fn FT_New_Memory_Face(
    library: *mut c_void,
    file_base: *const c_uchar,
    file_size: c_long,
    face_index: c_long,
    aface: *mut *mut FT_FaceRec,
) -> c_int {
    let _ = library;
    if file_base.is_null() || file_size <= 0 || aface.is_null() {
        return 1;
    }

    let font_bytes = unsafe { slice::from_raw_parts(file_base, file_size as usize) }.to_vec();
    init_face_from_bytes(font_bytes, face_index, aface)
}

#[no_mangle]
pub extern "C" fn FT_New_Face(
    library: *mut c_void,
    filepathname: *const c_char,
    face_index: c_long,
    aface: *mut *mut FT_FaceRec,
) -> c_int {
    let _ = library;
    if filepathname.is_null() || aface.is_null() {
        return 1;
    }

    let c_str = unsafe { CStr::from_ptr(filepathname) };
    let path = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return 1,
    };

    let font_bytes = match fs::read(path) {
        Ok(b) => b,
        Err(_) => return 2, // Cannot open resource
    };

    init_face_from_bytes(font_bytes, face_index, aface)
}

pub type FT_Stream_IoFunc = unsafe extern "C" fn(
    stream: *mut FT_StreamRec,
    offset: c_ulong,
    buffer: *mut c_uchar,
    count: c_ulong,
) -> c_ulong;

pub type FT_Stream_CloseFunc = unsafe extern "C" fn(
    stream: *mut FT_StreamRec,
);

#[repr(C)]
pub struct FT_StreamRec {
    pub base: *mut c_uchar,
    pub size: c_ulong,
    pub pos: c_ulong,
    pub descriptor: *mut c_void,
    pub pathname: *mut c_void,
    pub read: Option<FT_Stream_IoFunc>,
    pub close: Option<FT_Stream_CloseFunc>,
    pub memory: *mut c_void,
    pub cursor: *mut c_uchar,
    pub limit: *mut c_uchar,
}

#[repr(C)]
pub struct FT_Open_Args {
    pub flags: c_uint,
    pub memory_base: *const c_uchar,
    pub memory_size: c_long,
    pub pathname: *mut c_char,
    pub stream: *mut FT_StreamRec,
    pub driver: *mut c_void,
    pub num_params: c_int,
    pub params: *mut c_void,
}

pub const FT_OPEN_MEMORY: c_uint = 0x1;
pub const FT_OPEN_STREAM: c_uint = 0x2;
pub const FT_OPEN_PATHNAME: c_uint = 0x4;
pub const FT_OPEN_DRIVER: c_uint = 0x8;
pub const FT_OPEN_PARAMS: c_uint = 0x10;

unsafe fn read_stream_bytes(stream: *mut FT_StreamRec) -> Option<Vec<u8>> {
    if stream.is_null() {
        return None;
    }
    let size = (*stream).size as usize;
    if size == 0 || size > 500 * 1024 * 1024 {
        return None;
    }
    if !(*stream).base.is_null() {
        return Some(std::slice::from_raw_parts((*stream).base, size).to_vec());
    }
    if let Some(read_fn) = (*stream).read {
        let mut buf = vec![0u8; size];
        let read_bytes = read_fn(stream, 0, buf.as_mut_ptr(), (*stream).size);
        if read_bytes == 0 && size > 0 {
            return None;
        }
        buf.truncate(read_bytes as usize);
        return Some(buf);
    }
    None
}

#[no_mangle]
pub unsafe extern "C" fn FT_Open_Face(
    library: *mut c_void,
    args: *const FT_Open_Args,
    face_index: c_long,
    aface: *mut *mut FT_FaceRec,
) -> c_int {
    if args.is_null() || aface.is_null() {
        return 1;
    }
    if ((*args).flags & FT_OPEN_MEMORY) != 0 && !(*args).memory_base.is_null() && (*args).memory_size > 0 {
        return FT_New_Memory_Face(library, (*args).memory_base, (*args).memory_size, face_index, aface);
    }
    if ((*args).flags & FT_OPEN_PATHNAME) != 0 && !(*args).pathname.is_null() {
        return FT_New_Face(library, (*args).pathname, face_index, aface);
    }
    if ((*args).flags & FT_OPEN_STREAM) != 0 && !(*args).stream.is_null() {
        if let Some(bytes) = read_stream_bytes((*args).stream) {
            return init_face_from_bytes(bytes, face_index, aface);
        }
    }
    1
}

#[no_mangle]
pub unsafe extern "C" fn FT_Attach_File(_face: *mut FT_FaceRec, _filepathname: *const c_char) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Attach_Stream(_face: *mut FT_FaceRec, _parameters: *mut FT_Open_Args) -> c_int {
    0
}

fn init_face_from_bytes(
    font_bytes: Vec<u8>,
    face_index: c_long,
    aface: *mut *mut FT_FaceRec,
) -> c_int {
    let total_faces = honey_core::count_faces(&font_bytes) as c_long;

    if face_index < 0 {
        // FreeType specification: if face_index < 0, only return the number of faces
        let face = Box::new(FT_FaceRec {
            num_faces: total_faces,
            face_index: -1,
            face_flags: FT_FACE_FLAG_SCALABLE | FT_FACE_FLAG_HORIZONTAL,
            style_flags: 0,
            num_glyphs: 0,
            family_name: ptr::null_mut(),
            style_name: ptr::null_mut(),
            num_fixed_sizes: 0,
            available_sizes: ptr::null_mut(),
            num_charmaps: 0,
            charmaps: ptr::null_mut(),
            generic: [0; 2],
            bbox: FT_BBox::default(),
            units_per_EM: 1000,
            ascender: 800,
            descender: -200,
            height: 1000,
            max_advance_width: 1000,
            max_advance_height: 1000,
            underline_position: -100,
            underline_thickness: 50,
            glyph: ptr::null_mut(),
            size: ptr::null_mut(),
            charmap: ptr::null_mut(),
            driver: ptr::null_mut(),
            memory: ptr::null_mut(),
            stream: ptr::null_mut(),
            sizes_list: [0; 2],
            autohint: [0; 2],
            extensions: ptr::null_mut(),
            internal: ptr::null_mut(),
        });

        unsafe {
            *aface = Box::into_raw(face);
        }
        return 0; // FT_Err_Ok
    }

    let actual_index = (face_index.max(0) & 0xFFFF) as u32;
    let (num_glyphs, units_per_em, ascender, descender, family_name, style_name, is_italic, is_bold, is_mono, is_var, sfnt_names, os2_table, head_table, hhea_table, post_table, maxp_table) =
        match HoneyFace::from_slice(&font_bytes, actual_index) {
            Ok(f) => {
                let base_ptr = font_bytes.as_ptr() as usize;
                let mut names = Vec::new();
                for name in f.raw_face().names() {
                    let n_ptr = name.name.as_ptr() as usize;
                    if n_ptr >= base_ptr && n_ptr + name.name.len() <= base_ptr + font_bytes.len() {
                        names.push(SfntNameRecord {
                            platform_id: name.platform_id as c_ushort,
                            encoding_id: name.encoding_id,
                            language_id: name.language_id,
                            name_id: name.name_id,
                            offset: n_ptr - base_ptr,
                            len: name.name.len(),
                        });
                    }
                }
                let fam = f.family_name().unwrap_or_else(|| "Honey Font".to_string());
                let is_it = f.is_italic();
                let is_bld = f.is_bold();
                let sty = f.style_name().unwrap_or_else(|| {
                    if is_it && is_bld {
                        "Bold Italic".to_string()
                    } else if is_it {
                        "Italic".to_string()
                    } else if is_bld {
                        "Bold".to_string()
                    } else {
                        "Regular".to_string()
                    }
                });
                let os2 = f.table_data(*b"OS/2").and_then(parse_os2_table).map(Box::new);
                let head = f.table_data(*b"head").and_then(parse_head_table).map(Box::new);
                let hhea = f.table_data(*b"hhea").and_then(parse_hhea_table).map(Box::new);
                let post = f.table_data(*b"post").and_then(parse_post_table).map(Box::new);
                let maxp = f.table_data(*b"maxp").and_then(parse_maxp_table).map(Box::new);
                (
                    f.number_of_glyphs() as c_long,
                    f.units_per_em(),
                    f.ascender(),
                    f.descender(),
                    fam,
                    sty,
                    is_it,
                    is_bld,
                    f.is_monospaced(),
                    f.is_variable(),
                    names,
                    os2,
                    head,
                    hhea,
                    post,
                    maxp,
                )
            }
            Err(_) => return 3, // Unknown file format
        };

    let upem = units_per_em.max(1) as f32;
    let initial_scale_16 = ((16.0 / upem) * 65536.0) as c_long;

    let max_adv_w = hhea_table
        .as_ref()
        .map(|h| h.advance_Width_Max as c_short)
        .unwrap_or(units_per_em as c_short);

    let typo_asc = hhea_table
        .as_ref()
        .map(|h| h.Ascender)
        .unwrap_or(ascender);

    let typo_desc = hhea_table
        .as_ref()
        .map(|h| h.Descender)
        .unwrap_or(descender);

    let typo_gap = hhea_table.as_ref().map(|h| h.Line_Gap).unwrap_or(0);
    let face_h = typo_asc - typo_desc + typo_gap;

    let mut context = Box::new(FaceContext {
        font_data: font_bytes,
        actual_face_index: actual_index,
        pixel_size: 16.0,
        last_bitmap_buffer: Vec::new(),
        outline_points: Vec::new(),
        outline_tags: Vec::new(),
        outline_contours: Vec::new(),
        charmap_obj: FT_CharMapRec {
            face: ptr::null_mut(),
            encoding: 0x756e_6963, // FT_ENCODING_UNICODE ('unic')
            platform_id: 3,        // Windows / Unicode
            encoding_id: 1,
        },
        charmaps_array: Vec::new(),
        size_obj: FT_SizeRec {
            face: ptr::null_mut(),
            generic: [0; 2],
            metrics: FT_Size_Metrics {
                x_ppem: 16,
                y_ppem: 16,
                x_scale: initial_scale_16,
                y_scale: initial_scale_16,
                ascender: (((typo_asc as f32 / upem) * 16.0 * 64.0) as c_long),
                descender: (((typo_desc as f32 / upem) * 16.0 * 64.0) as c_long),
                height: (((face_h as f32 / upem) * 16.0 * 64.0) as c_long),
                max_advance: (((max_adv_w as f32 / upem) * 16.0 * 64.0) as c_long),
            },
            internal: ptr::null_mut(),
        },
        transform_matrix: None,
        transform_delta: None,
        sfnt_names,
        os2_table,
        head_table,
        hhea_table,
        post_table,
        maxp_table,
    });

    let slot = Box::new(FT_GlyphSlotRec {
        library: 0xCAFE_BABE as *mut c_void,
        face: ptr::null_mut(),
        next: ptr::null_mut(),
        glyph_index: 0,
        generic: [0; 2],
        metrics: FT_Glyph_Metrics::default(),
        linearHoriAdvance: 0,
        linearVertAdvance: 0,
        advance: FT_Vector::default(),
        format: FT_GLYPH_FORMAT_BITMAP,
        bitmap: FT_Bitmap {
            rows: 0,
            width: 0,
            pitch: 0,
            buffer: ptr::null_mut(),
            num_grays: 256,
            pixel_mode: FT_PIXEL_MODE_GRAY,
            palette_mode: 0,
            palette: ptr::null_mut(),
        },
        bitmap_left: 0,
        bitmap_top: 0,
        outline: FT_Outline {
            n_contours: 0,
            n_points: 0,
            points: ptr::null_mut(),
            tags: ptr::null_mut(),
            contours: ptr::null_mut(),
            flags: 0x01,
        },
        num_subglyphs: 0,
        subglyphs: ptr::null_mut(),
        control_data: ptr::null_mut(),
        control_len: 0,
        lsb_delta: 0,
        rsb_delta: 0,
        other: ptr::null_mut(),
        internal: ptr::null_mut(),
    });

    let mut computed_face_flags: c_long = FT_FACE_FLAG_SCALABLE | FT_FACE_FLAG_HORIZONTAL | FT_FACE_FLAG_SFNT | FT_FACE_FLAG_KERNING;
    if num_glyphs > 0 {
        computed_face_flags |= FT_FACE_FLAG_GLYPH_NAMES;
    }
    if is_mono {
        computed_face_flags |= FT_FACE_FLAG_FIXED_WIDTH;
    }
    if is_var {
        computed_face_flags |= FT_FACE_FLAG_MULTIPLE_MASTERS | FT_FACE_FLAG_VARIATION;
    }

    let mut computed_style_flags: c_long = 0;
    if is_italic || style_name.to_lowercase().contains("italic") || style_name.to_lowercase().contains("oblique") {
        computed_style_flags |= FT_STYLE_FLAG_ITALIC;
    }
    if is_bold || style_name.to_lowercase().contains("bold") {
        computed_style_flags |= FT_STYLE_FLAG_BOLD;
    }

    let family = CString::new(family_name).unwrap_or_else(|_| CString::new("Font").unwrap()).into_raw();
    let style = CString::new(style_name).unwrap_or_else(|_| CString::new("Regular").unwrap()).into_raw();
    let slot_raw = Box::into_raw(slot);

    context.charmaps_array.push(&mut context.charmap_obj as *mut FT_CharMapRec);

    let face = Box::new(FT_FaceRec {
        num_faces: total_faces,
        face_index,
        face_flags: computed_face_flags,
        style_flags: computed_style_flags,
        num_glyphs,
        family_name: family,
        style_name: style,
        num_fixed_sizes: 0,
        available_sizes: ptr::null_mut(),
        num_charmaps: 1,
        charmaps: context.charmaps_array.as_mut_ptr(),
        generic: [0; 2],
        bbox: FT_BBox {
            xMin: context.head_table.as_ref().map(|h| h.xMin as c_long).unwrap_or(0),
            yMin: context.head_table.as_ref().map(|h| h.yMin as c_long).unwrap_or(typo_desc as c_long),
            xMax: context.head_table.as_ref().map(|h| h.xMax as c_long).unwrap_or(units_per_em as c_long),
            yMax: context.head_table.as_ref().map(|h| h.yMax as c_long).unwrap_or(typo_asc as c_long),
        },
        units_per_EM: units_per_em,
        ascender: typo_asc,
        descender: typo_desc,
        height: face_h,
        max_advance_width: max_adv_w,
        max_advance_height: face_h,
        underline_position: context.post_table.as_ref().map(|p| p.underlinePosition).unwrap_or(-100),
        underline_thickness: context.post_table.as_ref().map(|p| p.underlineThickness).unwrap_or(50),
        glyph: slot_raw,
        size: &mut context.size_obj as *mut FT_SizeRec,
        charmap: &mut context.charmap_obj as *mut FT_CharMapRec,
        driver: ptr::null_mut(),
        memory: ptr::null_mut(),
        stream: ptr::null_mut(),
        sizes_list: [0; 2],
        autohint: [0; 2],
        extensions: ptr::null_mut(),
        internal: Box::into_raw(context),
    });

    let face_raw = Box::into_raw(face);
    unsafe {
        (*slot_raw).face = face_raw;
        let ctx = (*face_raw).internal;
        (*ctx).charmap_obj.face = face_raw;
        (*ctx).size_obj.face = face_raw;
        *aface = face_raw;
    }
    0
}

#[no_mangle]
pub extern "C" fn FT_Reference_Face(face: *mut FT_FaceRec) -> c_int {
    if face.is_null() {
        return 1;
    }
    0
}

#[no_mangle]
pub extern "C" fn FT_Done_Face(face: *mut FT_FaceRec) -> c_int {
    if face.is_null() {
        return 1;
    }
    unsafe {
        let face_box = Box::from_raw(face);
        if !face_box.glyph.is_null() {
            let _ = Box::from_raw(face_box.glyph);
        }
        if !face_box.internal.is_null() {
            let _ = Box::from_raw(face_box.internal as *mut FaceContext);
        }
        if !face_box.family_name.is_null() {
            let _ = CString::from_raw(face_box.family_name);
        }
        if !face_box.style_name.is_null() {
            let _ = CString::from_raw(face_box.style_name);
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn FT_Set_Pixel_Sizes(
    face: *mut FT_FaceRec,
    pixel_width: c_uint,
    pixel_height: c_uint,
) -> c_int {
    if face.is_null() {
        return 1;
    }
    let target = if pixel_height > 0 {
        pixel_height as f32
    } else {
        pixel_width as f32
    };

    unsafe {
        let ctx = (*face).internal;
        if !ctx.is_null() {
            let px = target.max(4.0);
            (*ctx).pixel_size = px;
            let upem = (*face).units_per_EM.max(1) as f32;
            let scale_16 = ((px / upem) * 65536.0) as c_long;
            (*ctx).size_obj.metrics.x_ppem = px as c_ushort;
            (*ctx).size_obj.metrics.y_ppem = px as c_ushort;
            (*ctx).size_obj.metrics.x_scale = scale_16;
            (*ctx).size_obj.metrics.y_scale = scale_16;
            (*ctx).size_obj.metrics.ascender = (((*face).ascender as f32 / upem) * px * 64.0) as c_long;
            (*ctx).size_obj.metrics.descender = (((*face).descender as f32 / upem) * px * 64.0) as c_long;
            (*ctx).size_obj.metrics.height = (((*face).height as f32 / upem) * px * 64.0) as c_long;
            (*ctx).size_obj.metrics.max_advance = (((*face).max_advance_width as f32 / upem) * px * 64.0) as c_long;
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn FT_Set_Char_Size(
    face: *mut FT_FaceRec,
    _char_width: c_long,
    char_height: c_long,
    _horz_res: c_uint,
    vert_res: c_uint,
) -> c_int {
    if face.is_null() {
        return 1;
    }
    let dpi = if vert_res > 0 { vert_res as f32 } else { 72.0 };
    let pt = (char_height as f32) / 64.0;
    let px = pt * (dpi / 72.0);

    unsafe {
        let ctx = (*face).internal;
        if !ctx.is_null() {
            let px_clamped = px.max(4.0);
            (*ctx).pixel_size = px_clamped;
            let upem = (*face).units_per_EM.max(1) as f32;
            let scale_16 = ((px_clamped / upem) * 65536.0) as c_long;
            (*ctx).size_obj.metrics.x_ppem = px_clamped as c_ushort;
            (*ctx).size_obj.metrics.y_ppem = px_clamped as c_ushort;
            (*ctx).size_obj.metrics.x_scale = scale_16;
            (*ctx).size_obj.metrics.y_scale = scale_16;
            (*ctx).size_obj.metrics.ascender = (((*face).ascender as f32 / upem) * px_clamped * 64.0) as c_long;
            (*ctx).size_obj.metrics.descender = (((*face).descender as f32 / upem) * px_clamped * 64.0) as c_long;
            (*ctx).size_obj.metrics.height = ((((*face).ascender - (*face).descender) as f32 / upem) * px_clamped * 64.0) as c_long;
            (*ctx).size_obj.metrics.max_advance = (((*face).max_advance_width as f32 / upem) * px_clamped * 64.0) as c_long;
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn FT_Get_Char_Index(face: *mut FT_FaceRec, charcode: c_ulong) -> c_uint {
    if face.is_null() {
        return 0;
    }
    unsafe {
        let ctx = (*face).internal;
        if ctx.is_null() {
            return 0;
        }
        if let Ok(hf) = HoneyFace::from_slice(&(*ctx).font_data, (*ctx).actual_face_index) {
            if let Some(ch) = char::from_u32(charcode as u32) {
                return hf.glyph_index(ch).unwrap_or(0);
            }
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_First_Char(face: *mut FT_FaceRec, agindex: *mut c_uint) -> c_ulong {
    if face.is_null() {
        if !agindex.is_null() {
            *agindex = 0;
        }
        return 0;
    }
    FT_Get_Next_Char(face, 0, agindex)
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Next_Char(
    face: *mut FT_FaceRec,
    char_code: c_ulong,
    agindex: *mut c_uint,
) -> c_ulong {
    if face.is_null() {
        if !agindex.is_null() {
            *agindex = 0;
        }
        return 0;
    }
    let ctx = (*face).internal;
    if ctx.is_null() {
        if !agindex.is_null() {
            *agindex = 0;
        }
        return 0;
    }
    if let Ok(hf) = HoneyFace::from_slice(&(*ctx).font_data, (*ctx).actual_face_index) {
        let start = (char_code as u32).saturating_add(1);
        for code in start..=0x10FFFF {
            if let Some(ch) = char::from_u32(code) {
                if let Some(gid) = hf.glyph_index(ch) {
                    if gid > 0 {
                        if !agindex.is_null() {
                            *agindex = gid;
                        }
                        return code as c_ulong;
                    }
                }
            }
        }
    }
    if !agindex.is_null() {
        *agindex = 0;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Load_Glyph(
    face: *mut FT_FaceRec,
    glyph_index: c_uint,
    load_flags: c_int,
) -> c_int {
    if face.is_null() {
        return 1;
    }
    let slot = (*face).glyph;
    if slot.is_null() {
        return 1;
    }
    let ctx = (*face).internal;
    if ctx.is_null() {
        return 1;
    }

    (*slot).glyph_index = glyph_index;
    let scale = (*ctx).pixel_size / ((*face).units_per_EM.max(1) as f32);

    if let Ok(hf) = HoneyFace::from_slice(&(*ctx).font_data, (*ctx).actual_face_index) {
        let mut accum = OutlineAccumulator::new(scale);
        let _ = hf.outline_glyph(glyph_index, &mut accum);
        accum.close();

        (*ctx).outline_points = accum.points;
        (*ctx).outline_tags = accum.tags;
        (*ctx).outline_contours = accum.contours;

        (*slot).outline.n_points = (*ctx).outline_points.len() as c_short;
        (*slot).outline.n_contours = (*ctx).outline_contours.len() as c_short;
        (*slot).outline.points = (*ctx).outline_points.as_mut_ptr();
        (*slot).outline.tags = (*ctx).outline_tags.as_mut_ptr();
        (*slot).outline.contours = (*ctx).outline_contours.as_mut_ptr();
        (*slot).outline.flags = 0x01;

        let metrics = hf.glyph_metrics(glyph_index, (*ctx).pixel_size);
        (*slot).metrics.width = (metrics.width as c_long) << 6;
        (*slot).metrics.height = (metrics.height as c_long) << 6;
        (*slot).metrics.horiBearingX = (metrics.left as c_long) << 6;
        (*slot).metrics.horiBearingY = (metrics.top as c_long) << 6;
        (*slot).metrics.horiAdvance = (metrics.advance * 64.0).round() as c_long;
        (*slot).linearHoriAdvance = (metrics.advance * 65536.0).round() as c_long;
        (*slot).advance.x = (metrics.advance * 64.0).round() as c_long;
        (*slot).advance.y = 0;
        (*slot).format = FT_GLYPH_FORMAT_OUTLINE;

        if (load_flags & FT_LOAD_RENDER) != 0 {
            return FT_Render_Glyph(slot, 0);
        }
        return 0;
    }
    1
}

#[no_mangle]
pub extern "C" fn FT_Load_Char(face: *mut FT_FaceRec, char_code: c_ulong, load_flags: c_int) -> c_int {
    let gid = FT_Get_Char_Index(face, char_code);
    unsafe { FT_Load_Glyph(face, gid, load_flags) }
}

#[no_mangle]
pub extern "C" fn FT_Render_Glyph(slot: *mut FT_GlyphSlotRec, _render_mode: c_uint) -> c_int {
    if slot.is_null() {
        return 1;
    }
    unsafe {
        let face = (*slot).face;
        if face.is_null() {
            return 1;
        }
        let ctx = (*face).internal;
        if ctx.is_null() {
            return 1;
        }

        let gid = (*slot).glyph_index;
        let px_size = (*ctx).pixel_size;
        let profile = get_active_profile();

        if let Ok(hf) = HoneyFace::from_slice(&(*ctx).font_data, (*ctx).actual_face_index) {
            if let Some((bmp, left, top, adv)) = hf.render_glyph_with_profile(gid, px_size, 0.0, 0.0, profile) {
                // Row stride/pitch MUST be divisible by 4 for Cairo/Pango image surfaces
                let pitch = ((bmp.width + 3) & !3) as usize;
                let height = bmp.height as usize;
                let padded_len = pitch * height;

                (*ctx).last_bitmap_buffer.clear();
                (*ctx).last_bitmap_buffer.resize(padded_len, 0);

                if bmp.width > 0 && height > 0 {
                    for y in 0..height {
                        let src_start = y * (bmp.width as usize);
                        let dst_start = y * pitch;
                        ptr::copy_nonoverlapping(
                            bmp.buffer.as_ptr().add(src_start),
                            (*ctx).last_bitmap_buffer.as_mut_ptr().add(dst_start),
                            bmp.width as usize,
                        );
                    }
                }

                (*slot).bitmap.width = bmp.width as c_uint;
                (*slot).bitmap.rows = bmp.height as c_uint;
                (*slot).bitmap.pitch = pitch as c_int;
                (*slot).bitmap.buffer = if padded_len > 0 {
                    (*ctx).last_bitmap_buffer.as_mut_ptr()
                } else {
                    ptr::null_mut()
                };
                (*slot).bitmap.num_grays = 256;
                (*slot).bitmap.pixel_mode = FT_PIXEL_MODE_GRAY;

                (*slot).bitmap_left = left;
                (*slot).bitmap_top = top;
                (*slot).advance.x = (adv * 64.0).round() as c_long;
                (*slot).advance.y = 0;
                (*slot).metrics.width = (bmp.width as c_long) << 6;
                (*slot).metrics.height = (bmp.height as c_long) << 6;
                (*slot).metrics.horiBearingX = (left as c_long) << 6;
                (*slot).metrics.horiBearingY = (top as c_long) << 6;
                (*slot).metrics.horiAdvance = (adv * 64.0).round() as c_long;
                (*slot).linearHoriAdvance = (adv * 65536.0).round() as c_long;
                (*slot).format = FT_GLYPH_FORMAT_BITMAP;

                return 0;
            } else {
                // Empty glyph (e.g. whitespace, control character, zero-contour outline)
                let upem = (*face).units_per_EM.max(1) as f32;
                let adv = hf.glyph_hor_advance(gid).unwrap_or(0) as f32 * (px_size / upem);
                (*ctx).last_bitmap_buffer.clear();
                (*slot).bitmap.width = 0;
                (*slot).bitmap.rows = 0;
                (*slot).bitmap.pitch = 0;
                (*slot).bitmap.buffer = ptr::null_mut();
                (*slot).bitmap.num_grays = 256;
                (*slot).bitmap.pixel_mode = FT_PIXEL_MODE_GRAY;
                (*slot).bitmap_left = 0;
                (*slot).bitmap_top = 0;
                (*slot).advance.x = (adv * 64.0).round() as c_long;
                (*slot).advance.y = 0;
                (*slot).metrics.width = 0;
                (*slot).metrics.height = 0;
                (*slot).metrics.horiBearingX = 0;
                (*slot).metrics.horiBearingY = 0;
                (*slot).metrics.horiAdvance = (adv * 64.0).round() as c_long;
                (*slot).linearHoriAdvance = (adv * 65536.0).round() as c_long;
                (*slot).format = FT_GLYPH_FORMAT_BITMAP;

                return 0;
            }
        }
    }
    1
}

// =========================================================================
// OpenType Tables & Metadata Queries (HarfBuzz, Fontconfig, Cairo)
// =========================================================================

#[no_mangle]
pub unsafe extern "C" fn FT_Load_Sfnt_Table(
    face: *mut FT_FaceRec,
    tag: c_ulong,
    offset: c_long,
    buffer: *mut c_uchar,
    length: *mut c_ulong,
) -> c_int {
    if face.is_null() || length.is_null() {
        return 1;
    }
    let ctx = (*face).internal;
    if ctx.is_null() {
        return 1;
    }
    let tag_u32 = tag as u32;
    let tag_bytes = tag_u32.to_be_bytes();

    if let Ok(hf) = HoneyFace::from_slice(&(*ctx).font_data, (*ctx).actual_face_index) {
        if let Some(table) = hf.table_data(tag_bytes) {
            let table_len = table.len();
            if buffer.is_null() {
                *length = table_len as c_ulong;
                return 0;
            }

            let off = if offset > 0 { offset as usize } else { 0 };
            if off >= table_len {
                *length = 0;
                return 0;
            }

            let available = table_len - off;
            let to_copy = (*length as usize).min(available);
            ptr::copy_nonoverlapping(table[off..].as_ptr(), buffer, to_copy);
            *length = to_copy as c_ulong;
            return 0;
        }
    }
    0x85 // FT_Err_Table_Missing
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Sfnt_Table(face: *mut FT_FaceRec, tag: c_uint) -> *mut c_void {
    if face.is_null() {
        return ptr::null_mut();
    }
    let ctx = match unsafe { (*face).internal.as_mut() } {
        Some(c) => c,
        None => return ptr::null_mut(),
    };
    match tag {
        FT_SFNT_HEAD => {
            if let Some(ref mut head) = ctx.head_table {
                return head.as_mut() as *mut TT_Header as *mut c_void;
            }
        }
        FT_SFNT_MAXP => {
            if let Some(ref mut maxp) = ctx.maxp_table {
                return maxp.as_mut() as *mut TT_MaxProfile as *mut c_void;
            }
        }
        FT_SFNT_OS2 => {
            if let Some(ref mut os2) = ctx.os2_table {
                return os2.as_mut() as *mut TT_OS2 as *mut c_void;
            }
        }
        FT_SFNT_HHEA => {
            if let Some(ref mut hhea) = ctx.hhea_table {
                return hhea.as_mut() as *mut TT_HoriHeader as *mut c_void;
            }
        }
        FT_SFNT_POST => {
            if let Some(ref mut post) = ctx.post_table {
                return post.as_mut() as *mut TT_Postscript as *mut c_void;
            }
        }
        _ => {}
    }
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn FT_Sfnt_Table_Info(
    face: *mut FT_FaceRec,
    table_index: c_uint,
    tag: *mut c_ulong,
    length: *mut c_ulong,
) -> c_int {
    if face.is_null() {
        return 1;
    }
    let ctx = (*face).internal;
    if ctx.is_null() {
        return 1;
    }
    let data = &(*ctx).font_data;
    if data.len() < 12 {
        return 1;
    }
    let num_tables = u16::from_be_bytes([data[4], data[5]]) as usize;
    let idx = table_index as usize;
    if idx >= num_tables {
        return 1;
    }
    let rec_off = 12 + idx * 16;
    if rec_off + 16 > data.len() {
        return 1;
    }
    let tag_val = u32::from_be_bytes([data[rec_off], data[rec_off + 1], data[rec_off + 2], data[rec_off + 3]]);
    let len_val = u32::from_be_bytes([data[rec_off + 12], data[rec_off + 13], data[rec_off + 14], data[rec_off + 15]]);

    if !tag.is_null() {
        *tag = tag_val as c_ulong;
    }
    if !length.is_null() {
        *length = len_val as c_ulong;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Select_Charmap(face: *mut FT_FaceRec, encoding: c_uint) -> c_int {
    if face.is_null() {
        return 1;
    }
    let ctx = (*face).internal;
    if ctx.is_null() {
        return 1;
    }
    (*ctx).charmap_obj.encoding = encoding;
    (*face).charmap = &mut (*ctx).charmap_obj;
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Set_Charmap(face: *mut FT_FaceRec, charmap: *mut FT_CharMapRec) -> c_int {
    if face.is_null() || charmap.is_null() {
        return 1;
    }
    (*face).charmap = charmap;
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Charmap_Index(charmap: *mut FT_CharMapRec) -> c_int {
    if charmap.is_null() {
        return -1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Kerning(
    face: *mut FT_FaceRec,
    left_glyph: c_uint,
    right_glyph: c_uint,
    _kern_mode: c_uint,
    akerning: *mut FT_Vector,
) -> c_int {
    if face.is_null() || akerning.is_null() {
        return 1;
    }
    (*akerning).x = 0;
    (*akerning).y = 0;

    let ctx = (*face).internal;
    if ctx.is_null() {
        return 0;
    }
    if let Ok(hf) = HoneyFace::from_slice(&(*ctx).font_data, (*ctx).actual_face_index) {
        if let Some(kern_data) = hf.table_data(*b"kern") {
            if kern_data.len() >= 18 {
                let n_pairs = u16::from_be_bytes([kern_data[10], kern_data[11]]) as usize;
                let mut pos = 18;
                for _ in 0..n_pairs {
                    if pos + 6 > kern_data.len() {
                        break;
                    }
                    let left = u16::from_be_bytes([kern_data[pos], kern_data[pos + 1]]) as c_uint;
                    let right = u16::from_be_bytes([kern_data[pos + 2], kern_data[pos + 3]]) as c_uint;
                    let val = i16::from_be_bytes([kern_data[pos + 4], kern_data[pos + 5]]);
                    if left == left_glyph && right == right_glyph {
                        let scale = (*ctx).pixel_size / (hf.units_per_em().max(1) as f32);
                        (*akerning).x = (val as f32 * scale * 64.0) as c_long;
                        return 0;
                    }
                    pos += 6;
                }
            }
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Advance(
    face: *mut FT_FaceRec,
    gindex: c_uint,
    _flags: c_int,
    padvance: *mut c_long,
) -> c_int {
    if face.is_null() || padvance.is_null() {
        return 1;
    }
    let ctx = (*face).internal;
    if ctx.is_null() {
        return 1;
    }
    if let Ok(hf) = HoneyFace::from_slice(&(*ctx).font_data, (*ctx).actual_face_index) {
        let upem = hf.units_per_em().max(1) as f32;
        let scale = (*ctx).pixel_size / upem;
        let adv = hf.glyph_hor_advance(gindex).unwrap_or(0) as f32 * scale;
        *padvance = (adv * 64.0) as c_long;
        return 0;
    }
    1
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Advances(
    face: *mut FT_FaceRec,
    start: c_uint,
    count: c_uint,
    flags: c_int,
    padvances: *mut c_long,
) -> c_int {
    if face.is_null() || padvances.is_null() {
        return 1;
    }
    for i in 0..count {
        FT_Get_Advance(face, start + i, flags, padvances.add(i as usize));
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Font_Format(face: *mut FT_FaceRec) -> *const c_char {
    let _ = face;
    static FORMAT: &[u8] = b"TrueType\0";
    FORMAT.as_ptr() as *const c_char
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_X11_Font_Format(face: *mut FT_FaceRec) -> *const c_char {
    FT_Get_Font_Format(face)
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Postscript_Name(face: *mut FT_FaceRec) -> *const c_char {
    if face.is_null() {
        return ptr::null();
    }
    let ctx = (*face).internal;
    if ctx.is_null() {
        return (*face).family_name;
    }
    if let Ok(hf) = HoneyFace::from_slice(&(*ctx).font_data, (*ctx).actual_face_index) {
        if let Some(ps) = hf.postscript_name() {
            if let Ok(c_ps) = CString::new(ps) {
                return c_ps.into_raw();
            }
        }
    }
    (*face).family_name
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Glyph_Name(
    face: *mut FT_FaceRec,
    glyph_index: c_uint,
    buffer: *mut c_char,
    buffer_max: c_uint,
) -> c_int {
    if face.is_null() || buffer.is_null() || buffer_max == 0 {
        return 1;
    }
    let ctx = (*face).internal;
    if ctx.is_null() {
        return 1;
    }
    if let Ok(hf) = HoneyFace::from_slice(&(*ctx).font_data, (*ctx).actual_face_index) {
        if let Some(name) = hf.glyph_name(glyph_index) {
            let bytes = name.as_bytes();
            let len = bytes.len().min(buffer_max as usize - 1);
            ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, len);
            *buffer.add(len) = 0;
            return 0;
        }
    }
    let fallback = format!("g{}\0", glyph_index);
    let bytes = fallback.as_bytes();
    let len = bytes.len().min(buffer_max as usize);
    ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, len);
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Name_Index(
    _face: *mut FT_FaceRec,
    glyph_name: *const c_char,
) -> c_uint {
    if glyph_name.is_null() {
        return 0;
    }
    let c_str = CStr::from_ptr(glyph_name);
    if let Ok(s) = c_str.to_str() {
        if let Some(rest) = s.strip_prefix('g') {
            if let Ok(id) = rest.parse::<c_uint>() {
                return id;
            }
        }
    }
    0
}

#[repr(C)]
pub struct FT_SfntName {
    pub platform_id: c_ushort,
    pub encoding_id: c_ushort,
    pub language_id: c_ushort,
    pub name_id: c_ushort,
    pub string: *mut c_uchar,
    pub string_len: c_uint,
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Sfnt_Name_Count(face: *mut FT_FaceRec) -> c_uint {
    if face.is_null() {
        return 0;
    }
    let ctx = match unsafe { (*face).internal.as_ref() } {
        Some(c) => c,
        None => return 0,
    };
    ctx.sfnt_names.len() as c_uint
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Sfnt_Name(
    face: *mut FT_FaceRec,
    idx: c_uint,
    aname: *mut FT_SfntName,
) -> c_int {
    if face.is_null() || aname.is_null() {
        return 1;
    }
    let ctx = match unsafe { (*face).internal.as_ref() } {
        Some(c) => c,
        None => return 1,
    };
    let idx_usize = idx as usize;
    if idx_usize >= ctx.sfnt_names.len() {
        return 6; // FT_Err_Invalid_Argument
    }
    let rec = &ctx.sfnt_names[idx_usize];
    (*aname).platform_id = rec.platform_id;
    (*aname).encoding_id = rec.encoding_id;
    (*aname).language_id = rec.language_id;
    (*aname).name_id = rec.name_id;
    (*aname).string = ctx.font_data[rec.offset..].as_ptr() as *mut c_uchar;
    (*aname).string_len = rec.len as c_uint;
    0
}

// =========================================================================
// Outlines Decomposition & Vector Geometries (Cairo, Skia)
// =========================================================================

#[no_mangle]
pub unsafe extern "C" fn FT_Outline_Decompose(
    outline: *mut FT_Outline,
    func_interface: *const FT_Outline_Funcs,
    user_data: *mut c_void,
) -> c_int {
    if outline.is_null() || func_interface.is_null() {
        return 1;
    }
    let n_contours = (*outline).n_contours as usize;
    let n_points = (*outline).n_points as usize;
    if n_contours == 0 || n_points == 0 {
        return 0;
    }
    let points = (*outline).points;
    let tags = (*outline).tags;
    let contours = (*outline).contours;
    if points.is_null() || tags.is_null() || contours.is_null() {
        return 1;
    }

    let move_to = (*func_interface).move_to;
    let line_to = (*func_interface).line_to;
    let conic_to = (*func_interface).conic_to;
    let cubic_to = (*func_interface).cubic_to;

    let mut start_idx = 0usize;
    for c in 0..n_contours {
        let end_idx = *contours.add(c) as usize;
        if end_idx >= n_points || start_idx > end_idx {
            break;
        }

        let first_pt = points.add(start_idx);
        if let Some(f) = move_to {
            let res = f(first_pt, user_data);
            if res != 0 {
                return res;
            }
        }

        let mut idx = start_idx + 1;
        while idx <= end_idx {
            let tag = (*tags.add(idx) as u8) & 0x03;
            if tag == 1 {
                if let Some(f) = line_to {
                    let res = f(points.add(idx), user_data);
                    if res != 0 {
                        return res;
                    }
                }
                idx += 1;
            } else if tag == 0 {
                let ctrl = points.add(idx);
                idx += 1;
                let to = if idx <= end_idx { points.add(idx) } else { first_pt };
                if let Some(f) = conic_to {
                    let res = f(ctrl, to, user_data);
                    if res != 0 {
                        return res;
                    }
                }
                idx += 1;
            } else if tag == 2 {
                let ctrl1 = points.add(idx);
                idx += 1;
                if idx <= end_idx {
                    let ctrl2 = points.add(idx);
                    idx += 1;
                    let to = if idx <= end_idx { points.add(idx) } else { first_pt };
                    if let Some(f) = cubic_to {
                        let res = f(ctrl1, ctrl2, to, user_data);
                        if res != 0 {
                            return res;
                        }
                    }
                    idx += 1;
                }
            } else {
                idx += 1;
            }
        }
        start_idx = end_idx + 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Outline_Get_BBox(
    outline: *mut FT_Outline,
    abbox: *mut FT_BBox,
) -> c_int {
    if outline.is_null() || abbox.is_null() {
        return 1;
    }
    let n_points = (*outline).n_points as usize;
    if n_points == 0 {
        (*abbox).xMin = 0;
        (*abbox).yMin = 0;
        (*abbox).xMax = 0;
        (*abbox).yMax = 0;
        return 0;
    }
    let pts = (*outline).points;
    if pts.is_null() {
        return 1;
    }
    let mut min_x = (*pts).x;
    let mut max_x = (*pts).x;
    let mut min_y = (*pts).y;
    let mut max_y = (*pts).y;
    for i in 1..n_points {
        let p = pts.add(i);
        if (*p).x < min_x { min_x = (*p).x; }
        if (*p).x > max_x { max_x = (*p).x; }
        if (*p).y < min_y { min_y = (*p).y; }
        if (*p).y > max_y { max_y = (*p).y; }
    }
    (*abbox).xMin = min_x;
    (*abbox).yMin = min_y;
    (*abbox).xMax = max_x;
    (*abbox).yMax = max_y;
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Outline_Get_CBox(
    outline: *mut FT_Outline,
    acbox: *mut FT_BBox,
) -> c_int {
    FT_Outline_Get_BBox(outline, acbox)
}

#[no_mangle]
pub unsafe extern "C" fn FT_Outline_Transform(
    outline: *mut FT_Outline,
    matrix: *const FT_Matrix,
) {
    if outline.is_null() || matrix.is_null() {
        return;
    }
    let n_points = (*outline).n_points as usize;
    let pts = (*outline).points;
    if pts.is_null() {
        return;
    }
    let xx = (*matrix).xx;
    let xy = (*matrix).xy;
    let yx = (*matrix).yx;
    let yy = (*matrix).yy;

    for i in 0..n_points {
        let p = pts.add(i);
        let x = (*p).x;
        let y = (*p).y;
        (*p).x = ((x * xx + y * xy) >> 16) as c_long;
        (*p).y = ((x * yx + y * yy) >> 16) as c_long;
    }
}

#[no_mangle]
pub unsafe extern "C" fn FT_Outline_Translate(
    outline: *mut FT_Outline,
    xOffset: c_long,
    yOffset: c_long,
) {
    if outline.is_null() {
        return;
    }
    let n_points = (*outline).n_points as usize;
    let pts = (*outline).points;
    if pts.is_null() {
        return;
    }
    for i in 0..n_points {
        let p = pts.add(i);
        (*p).x += xOffset;
        (*p).y += yOffset;
    }
}

#[no_mangle]
pub unsafe extern "C" fn FT_Outline_EmboldenXY(
    outline: *mut FT_Outline,
    _xstrength: c_long,
    _ystrength: c_long,
) -> c_int {
    if outline.is_null() {
        return 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Outline_Embolden(
    outline: *mut FT_Outline,
    strength: c_long,
) -> c_int {
    FT_Outline_EmboldenXY(outline, strength, strength)
}

#[no_mangle]
pub unsafe extern "C" fn FT_Outline_Get_Bitmap(
    _library: *mut c_void,
    outline: *mut FT_Outline,
    abitmap: *const FT_Bitmap,
) -> c_int {
    if outline.is_null() || abitmap.is_null() {
        return 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Outline_New(
    _library: *mut c_void,
    numPoints: c_uint,
    numContours: c_uint,
    anoutline: *mut FT_Outline,
) -> c_int {
    if anoutline.is_null() {
        return 1;
    }
    let pts_buf = vec![FT_Vector::default(); numPoints as usize].into_boxed_slice();
    let tags_buf = vec![0 as c_char; numPoints as usize].into_boxed_slice();
    let contours_buf = vec![0 as c_short; numContours as usize].into_boxed_slice();

    *anoutline = FT_Outline {
        n_contours: numContours as c_short,
        n_points: numPoints as c_short,
        points: Box::into_raw(pts_buf) as *mut FT_Vector,
        tags: Box::into_raw(tags_buf) as *mut c_char,
        contours: Box::into_raw(contours_buf) as *mut c_short,
        flags: 0x01,
    };
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Outline_Done(_library: *mut c_void, outline: *mut FT_Outline) -> c_int {
    if !outline.is_null() {
        if !(*outline).points.is_null() {
            let slice = slice::from_raw_parts_mut((*outline).points, (*outline).n_points as usize);
            let _ = Box::from_raw(slice as *mut [FT_Vector]);
            (*outline).points = ptr::null_mut();
        }
        if !(*outline).tags.is_null() {
            let slice = slice::from_raw_parts_mut((*outline).tags, (*outline).n_points as usize);
            let _ = Box::from_raw(slice as *mut [c_char]);
            (*outline).tags = ptr::null_mut();
        }
        if !(*outline).contours.is_null() {
            let slice = slice::from_raw_parts_mut((*outline).contours, (*outline).n_contours as usize);
            let _ = Box::from_raw(slice as *mut [c_short]);
            (*outline).contours = ptr::null_mut();
        }
    }
    0
}

// =========================================================================
// Independent Glyph Objects (FT_Glyph)
// =========================================================================

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Glyph(
    slot: *mut FT_GlyphSlotRec,
    aglyph: *mut *mut FT_GlyphRec,
) -> c_int {
    if slot.is_null() || aglyph.is_null() {
        return 1;
    }
    if (*slot).format == FT_GLYPH_FORMAT_BITMAP {
        let bmp_len = ((*slot).bitmap.rows * (*slot).bitmap.pitch.unsigned_abs() as u32) as usize;
        let buf_copy = if bmp_len > 0 && !(*slot).bitmap.buffer.is_null() {
            let ptr = calloc(bmp_len, 1) as *mut u8;
            if !ptr.is_null() {
                ptr::copy_nonoverlapping((*slot).bitmap.buffer, ptr, bmp_len);
            }
            ptr
        } else {
            ptr::null_mut()
        };

        let bg = Box::new(FT_BitmapGlyphRec {
            root: FT_GlyphRec {
                library: (*slot).library,
                clazz: ptr::null(),
                format: FT_GLYPH_FORMAT_BITMAP,
                advance: (*slot).advance,
            },
            left: (*slot).bitmap_left,
            top: (*slot).bitmap_top,
            bitmap: FT_Bitmap {
                rows: (*slot).bitmap.rows,
                width: (*slot).bitmap.width,
                pitch: (*slot).bitmap.pitch,
                buffer: buf_copy,
                num_grays: (*slot).bitmap.num_grays,
                pixel_mode: (*slot).bitmap.pixel_mode,
                palette_mode: 0,
                palette: ptr::null_mut(),
            },
        });
        *aglyph = Box::into_raw(bg) as *mut FT_GlyphRec;
        return 0;
    }

    let n_pts = (*slot).outline.n_points as usize;
    let n_cont = (*slot).outline.n_contours as usize;
    let (pts_copy, tags_copy, cont_copy) = if n_pts > 0 && !(*slot).outline.points.is_null() {
        let p_buf = calloc(n_pts, std::mem::size_of::<FT_Vector>()) as *mut FT_Vector;
        let t_buf = calloc(n_pts, std::mem::size_of::<c_char>()) as *mut c_char;
        let c_buf = calloc(n_cont, std::mem::size_of::<c_short>()) as *mut c_short;
        if !p_buf.is_null() {
            ptr::copy_nonoverlapping((*slot).outline.points, p_buf, n_pts);
        }
        if !t_buf.is_null() {
            ptr::copy_nonoverlapping((*slot).outline.tags, t_buf, n_pts);
        }
        if !c_buf.is_null() && n_cont > 0 && !(*slot).outline.contours.is_null() {
            ptr::copy_nonoverlapping((*slot).outline.contours, c_buf, n_cont);
        }
        (p_buf, t_buf, c_buf)
    } else {
        (ptr::null_mut(), ptr::null_mut(), ptr::null_mut())
    };

    let og = Box::new(FT_OutlineGlyphRec {
        root: FT_GlyphRec {
            library: (*slot).library,
            clazz: ptr::null(),
            format: FT_GLYPH_FORMAT_OUTLINE,
            advance: (*slot).advance,
        },
        outline: FT_Outline {
            n_contours: (*slot).outline.n_contours,
            n_points: (*slot).outline.n_points,
            points: pts_copy,
            tags: tags_copy,
            contours: cont_copy,
            flags: (*slot).outline.flags,
        },
    });
    *aglyph = Box::into_raw(og) as *mut FT_GlyphRec;
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Glyph_To_Bitmap(
    the_glyph: *mut *mut FT_GlyphRec,
    _render_mode: c_uint,
    origin: *mut FT_Vector,
    destroy: c_int,
) -> c_int {
    if the_glyph.is_null() || (*the_glyph).is_null() {
        return 1;
    }
    let glyph = *the_glyph;
    if (*glyph).format == FT_GLYPH_FORMAT_BITMAP {
        return 0;
    }
    if (*glyph).format != FT_GLYPH_FORMAT_OUTLINE {
        return 1;
    }

    let og = glyph as *mut FT_OutlineGlyphRec;
    let mut outline = (*og).outline;

    if !origin.is_null() {
        FT_Outline_Translate(&mut outline, (*origin).x, (*origin).y);
    }

    let mut cbox = FT_BBox { xMin: 0, yMin: 0, xMax: 0, yMax: 0 };
    FT_Outline_Get_CBox(&mut outline, &mut cbox);

    let n_points = outline.n_points as usize;
    if n_points == 0 {
        let bg = Box::new(FT_BitmapGlyphRec {
            root: FT_GlyphRec {
                library: (*glyph).library,
                clazz: ptr::null(),
                format: FT_GLYPH_FORMAT_BITMAP,
                advance: (*glyph).advance,
            },
            left: 0,
            top: 0,
            bitmap: FT_Bitmap {
                rows: 0,
                width: 0,
                pitch: 0,
                buffer: ptr::null_mut(),
                num_grays: 256,
                pixel_mode: FT_PIXEL_MODE_GRAY,
                palette_mode: 0,
                palette: ptr::null_mut(),
            },
        });
        if destroy != 0 {
            FT_Done_Glyph(glyph);
        }
        *the_glyph = Box::into_raw(bg) as *mut FT_GlyphRec;
        return 0;
    }

    let left = (cbox.xMin >> 6) as i32;
    let top = ((cbox.yMax + 63) >> 6) as i32;
    let right = ((cbox.xMax + 63) >> 6) as i32;
    let bottom = (cbox.yMin >> 6) as i32;
    let width = (right - left).max(0) as usize;
    let height = (top - bottom).max(0) as usize;
    let pitch = ((width + 3) & !3) as usize;

    let total_bytes = pitch * height;
    let buf_ptr = if total_bytes > 0 {
        calloc(total_bytes, 1) as *mut u8
    } else {
        ptr::null_mut()
    };

    let mut glyph_outline = GlyphOutline::new();
    let pts = outline.points;
    let tags = outline.tags;
    let contours = outline.contours;
    let n_contours = outline.n_contours as usize;

    let left_f = left as f32;
    let top_f = top as f32;

    let mut start_idx = 0usize;
    for c in 0..n_contours {
        let end_idx = *contours.add(c) as usize;
        if end_idx >= n_points || start_idx > end_idx {
            break;
        }

        let p0 = pts.add(start_idx);
        let px0 = ((*p0).x as f32 / 64.0) - left_f;
        let py0 = top_f - ((*p0).y as f32 / 64.0);
        glyph_outline.move_to(px0, py0);

        let mut i = start_idx + 1;
        while i <= end_idx {
            let p = pts.add(i);
            let tag = *tags.add(i);
            let px = ((*p).x as f32 / 64.0) - left_f;
            let py = top_f - ((*p).y as f32 / 64.0);

            if (tag & 1) == 1 {
                glyph_outline.line_to(px, py);
                i += 1;
            } else if (tag & 3) == 2 {
                // Cubic off-curve
                if i + 1 <= end_idx {
                    let p2 = pts.add(i + 1);
                    let p3 = if i + 2 <= end_idx { pts.add(i + 2) } else { pts.add(start_idx) };
                    let c1x = px;
                    let c1y = py;
                    let c2x = ((*p2).x as f32 / 64.0) - left_f;
                    let c2y = top_f - ((*p2).y as f32 / 64.0);
                    let to_x = ((*p3).x as f32 / 64.0) - left_f;
                    let to_y = top_f - ((*p3).y as f32 / 64.0);
                    glyph_outline.cubic_to(c1x, c1y, c2x, c2y, to_x, to_y);
                    i += 3;
                } else {
                    glyph_outline.line_to(px, py);
                    i += 1;
                }
            } else {
                // Conic quadratic off-curve
                let to = if i + 1 <= end_idx { pts.add(i + 1) } else { pts.add(start_idx) };
                let to_x = ((*to).x as f32 / 64.0) - left_f;
                let to_y = top_f - ((*to).y as f32 / 64.0);
                glyph_outline.quad_to(px, py, to_x, to_y);
                i += 2;
            }
        }
        glyph_outline.close();
        start_idx = end_idx + 1;
    }

    if width > 0 && height > 0 && !buf_ptr.is_null() {
        let rasterizer = QuartzRasterizer::new(width, height);
        let base_bmp = rasterizer.rasterize(&glyph_outline);
        let profile = get_active_profile();
        let factor = compute_dilation_factor_with_profile(12.0, profile);
        let dilated = dilate_mask(&base_bmp.buffer, width, height, factor);
        for y in 0..height {
            let src_off = y * width;
            let dst_off = y * pitch;
            ptr::copy_nonoverlapping(dilated.as_ptr().add(src_off), buf_ptr.add(dst_off), width);
        }
    }

    let bg = Box::new(FT_BitmapGlyphRec {
        root: FT_GlyphRec {
            library: (*glyph).library,
            clazz: ptr::null(),
            format: FT_GLYPH_FORMAT_BITMAP,
            advance: (*glyph).advance,
        },
        left,
        top,
        bitmap: FT_Bitmap {
            rows: height as c_uint,
            width: width as c_uint,
            pitch: pitch as c_int,
            buffer: buf_ptr,
            num_grays: 256,
            pixel_mode: FT_PIXEL_MODE_GRAY,
            palette_mode: 0,
            palette: ptr::null_mut(),
        },
    });

    if destroy != 0 {
        FT_Done_Glyph(glyph);
    }
    *the_glyph = Box::into_raw(bg) as *mut FT_GlyphRec;
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Done_Glyph(glyph: *mut FT_GlyphRec) {
    if glyph.is_null() {
        return;
    }
    if (*glyph).format == FT_GLYPH_FORMAT_BITMAP {
        let bg = glyph as *mut FT_BitmapGlyphRec;
        if !(*bg).bitmap.buffer.is_null() {
            free((*bg).bitmap.buffer as *mut c_void);
            (*bg).bitmap.buffer = ptr::null_mut();
        }
        let _ = Box::from_raw(bg);
    } else {
        let og = glyph as *mut FT_OutlineGlyphRec;
        if !(*og).outline.points.is_null() {
            free((*og).outline.points as *mut c_void);
            (*og).outline.points = ptr::null_mut();
        }
        if !(*og).outline.tags.is_null() {
            free((*og).outline.tags as *mut c_void);
            (*og).outline.tags = ptr::null_mut();
        }
        if !(*og).outline.contours.is_null() {
            free((*og).outline.contours as *mut c_void);
            (*og).outline.contours = ptr::null_mut();
        }
        let _ = Box::from_raw(og);
    }
}

#[no_mangle]
pub unsafe extern "C" fn FT_Glyph_Copy(
    source: *mut FT_GlyphRec,
    target: *mut *mut FT_GlyphRec,
) -> c_int {
    if source.is_null() || target.is_null() {
        return 1;
    }
    FT_Get_Glyph(source as *mut FT_GlyphSlotRec, target)
}

#[no_mangle]
pub unsafe extern "C" fn FT_Glyph_Transform(
    glyph: *mut FT_GlyphRec,
    matrix: *mut FT_Matrix,
    delta: *mut FT_Vector,
) -> c_int {
    if glyph.is_null() {
        return 1;
    }
    if (*glyph).format != FT_GLYPH_FORMAT_BITMAP {
        let og = glyph as *mut FT_OutlineGlyphRec;
        if !matrix.is_null() {
            FT_Outline_Transform(&mut (*og).outline, matrix);
        }
        if !delta.is_null() {
            FT_Outline_Translate(&mut (*og).outline, (*delta).x, (*delta).y);
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Glyph_Get_CBox(
    glyph: *mut FT_GlyphRec,
    _bbox_mode: c_uint,
    acbox: *mut FT_BBox,
) {
    if glyph.is_null() || acbox.is_null() {
        return;
    }
    if (*glyph).format == FT_GLYPH_FORMAT_BITMAP {
        let bg = glyph as *mut FT_BitmapGlyphRec;
        (*acbox).xMin = ((*bg).left as c_long) << 6;
        (*acbox).yMax = ((*bg).top as c_long) << 6;
        (*acbox).xMax = (*acbox).xMin + (((*bg).bitmap.width as c_long) << 6);
        (*acbox).yMin = (*acbox).yMax - (((*bg).bitmap.rows as c_long) << 6);
    } else {
        let og = glyph as *mut FT_OutlineGlyphRec;
        FT_Outline_Get_CBox(&mut (*og).outline, acbox);
    }
}

// =========================================================================
// Transforms, Geometry & Math
// =========================================================================

#[no_mangle]
pub unsafe extern "C" fn FT_Set_Transform(
    face: *mut FT_FaceRec,
    matrix: *mut FT_Matrix,
    delta: *mut FT_Vector,
) {
    if face.is_null() {
        return;
    }
    let ctx = (*face).internal;
    if !ctx.is_null() {
        (*ctx).transform_matrix = if !matrix.is_null() { Some(*matrix) } else { None };
        (*ctx).transform_delta = if !delta.is_null() { Some(*delta) } else { None };
    }
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Transform(
    face: *mut FT_FaceRec,
    matrix: *mut FT_Matrix,
    delta: *mut FT_Vector,
) {
    if face.is_null() {
        return;
    }
    let ctx = (*face).internal;
    if !ctx.is_null() {
        if !matrix.is_null() {
            *matrix = (*ctx).transform_matrix.unwrap_or(FT_Matrix { xx: 0x10000, xy: 0, yx: 0, yy: 0x10000 });
        }
        if !delta.is_null() {
            *delta = (*ctx).transform_delta.unwrap_or(FT_Vector { x: 0, y: 0 });
        }
    }
}

#[no_mangle]
pub extern "C" fn FT_MulFix(a: c_long, b: c_long) -> c_long {
    let prod = (a as i128) * (b as i128);
    let rounded = prod + 0x8000;
    (rounded >> 16) as c_long
}

#[no_mangle]
pub unsafe extern "C" fn FT_Vector_Transform(vec: *mut FT_Vector, matrix: *const FT_Matrix) {
    if vec.is_null() || matrix.is_null() {
        return;
    }
    let x = (*vec).x;
    let y = (*vec).y;
    let xx = (*matrix).xx;
    let xy = (*matrix).xy;
    let yx = (*matrix).yx;
    let yy = (*matrix).yy;
    (*vec).x = ((x * xx + y * xy + 0x8000) >> 16) as c_long;
    (*vec).y = ((x * yx + y * yy + 0x8000) >> 16) as c_long;
}

// =========================================================================
// Library Version, Modules & Properties
// =========================================================================

#[no_mangle]
pub unsafe extern "C" fn FT_Library_Version(
    _library: *mut c_void,
    amajor: *mut c_int,
    aminor: *mut c_int,
    apatch: *mut c_int,
) {
    if !amajor.is_null() { *amajor = 2; }
    if !aminor.is_null() { *aminor = 13; }
    if !apatch.is_null() { *apatch = 2; }
}

#[no_mangle]
pub unsafe extern "C" fn FT_Property_Set(
    _library: *mut c_void,
    _module_name: *const c_char,
    _property_name: *const c_char,
    _value: *const c_void,
) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Property_Get(
    _library: *mut c_void,
    _module_name: *const c_char,
    _property_name: *const c_char,
    _value: *mut c_void,
) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Library_SetLcdFilter(_library: *mut c_void, _filter: c_uint) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Library_SetLcdFilterWeights(
    _library: *mut c_void,
    _weights: *mut c_uchar,
) -> c_int {
    0
}

// =========================================================================
// Sizes & Strikes
// =========================================================================

#[no_mangle]
pub unsafe extern "C" fn FT_New_Size(
    face: *mut FT_FaceRec,
    asize: *mut *mut FT_SizeRec,
) -> c_int {
    if face.is_null() || asize.is_null() {
        return 1;
    }
    let sz = Box::new(FT_SizeRec {
        face,
        generic: [0; 2],
        metrics: FT_Size_Metrics {
            x_ppem: 16,
            y_ppem: 16,
            x_scale: 0x10000,
            y_scale: 0x10000,
            ascender: 16 << 6,
            descender: -4 << 6,
            height: 20 << 6,
            max_advance: 16 << 6,
        },
        internal: ptr::null_mut(),
    });
    *asize = Box::into_raw(sz);
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Done_Size(size: *mut FT_SizeRec) -> c_int {
    if !size.is_null() {
        let _ = Box::from_raw(size);
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Activate_Size(size: *mut FT_SizeRec) -> c_int {
    if size.is_null() {
        return 1;
    }
    let face = (*size).face;
    if !face.is_null() {
        (*face).size = size;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Select_Size(_face: *mut FT_FaceRec, _strike_index: c_int) -> c_int {
    0
}

// =========================================================================
// Bitmap Operations & Emboldening (Cairo)
// =========================================================================

#[no_mangle]
pub unsafe extern "C" fn FT_Bitmap_New(abitmap: *mut FT_Bitmap) {
    if abitmap.is_null() {
        return;
    }
    *abitmap = FT_Bitmap {
        rows: 0,
        width: 0,
        pitch: 0,
        buffer: ptr::null_mut(),
        num_grays: 256,
        pixel_mode: FT_PIXEL_MODE_GRAY,
        palette_mode: 0,
        palette: ptr::null_mut(),
    };
}

#[no_mangle]
pub unsafe extern "C" fn FT_Bitmap_Done(
    _library: *mut c_void,
    bitmap: *mut FT_Bitmap,
) -> c_int {
    if !bitmap.is_null() && !(*bitmap).buffer.is_null() {
        free((*bitmap).buffer as *mut c_void);
        (*bitmap).buffer = ptr::null_mut();
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Bitmap_Convert(
    _library: *mut c_void,
    source: *const FT_Bitmap,
    target: *mut FT_Bitmap,
    alignment: c_int,
) -> c_int {
    if source.is_null() || target.is_null() {
        return 1;
    }
    let align = if alignment > 0 { alignment as usize } else { 1 };
    let src_w = (*source).width as usize;
    let src_h = (*source).rows as usize;

    if src_w == 0 || src_h == 0 {
        *target = FT_Bitmap {
            rows: 0,
            width: 0,
            pitch: 0,
            buffer: ptr::null_mut(),
            num_grays: 256,
            pixel_mode: FT_PIXEL_MODE_GRAY,
            palette_mode: 0,
            palette: ptr::null_mut(),
        };
        return 0;
    }

    let src_pitch = (*source).pitch.unsigned_abs() as usize;

    match (*source).pixel_mode {
        FT_PIXEL_MODE_MONO => {
            // Unpack 1-bit monochrome (1 bit/pixel) into 8-bit grayscale (0x00 or 0xFF)
            let target_pitch = ((src_w + align - 1) / align) * align;
            let target_len = target_pitch * src_h;
            let ptr = calloc(target_len, 1) as *mut u8;
            if ptr.is_null() {
                return 1;
            }
            if !(*source).buffer.is_null() {
                for y in 0..src_h {
                    let src_row = (*source).buffer.add(y * src_pitch);
                    let dst_row = ptr.add(y * target_pitch);
                    for x in 0..src_w {
                        let byte = *src_row.add(x / 8);
                        let bit = (byte >> (7 - (x % 8))) & 1;
                        *dst_row.add(x) = if bit != 0 { 0xFF } else { 0x00 };
                    }
                }
            }
            *target = FT_Bitmap {
                rows: (*source).rows,
                width: (*source).width,
                pitch: target_pitch as c_int,
                buffer: ptr,
                num_grays: 256,
                pixel_mode: FT_PIXEL_MODE_GRAY,
                palette_mode: 0,
                palette: ptr::null_mut(),
            };
            0
        }
        FT_PIXEL_MODE_BGRA => {
            // 4 bytes per pixel color bitmap
            let row_bytes = src_w * 4;
            let target_pitch = ((row_bytes + align - 1) / align) * align;
            let target_len = target_pitch * src_h;
            let ptr = calloc(target_len, 1) as *mut u8;
            if ptr.is_null() {
                return 1;
            }
            if !(*source).buffer.is_null() {
                for y in 0..src_h {
                    let s_offset = y * src_pitch;
                    let d_offset = y * target_pitch;
                    let copy_w = row_bytes.min(src_pitch);
                    ptr::copy_nonoverlapping((*source).buffer.add(s_offset), ptr.add(d_offset), copy_w);
                }
            }
            *target = FT_Bitmap {
                rows: (*source).rows,
                width: (*source).width,
                pitch: target_pitch as c_int,
                buffer: ptr,
                num_grays: (*source).num_grays,
                pixel_mode: FT_PIXEL_MODE_BGRA,
                palette_mode: 0,
                palette: ptr::null_mut(),
            };
            0
        }
        _ => {
            // FT_PIXEL_MODE_GRAY or other 8-bit grayscale modes
            let target_pitch = ((src_w + align - 1) / align) * align;
            let target_len = target_pitch * src_h;
            let ptr = calloc(target_len, 1) as *mut u8;
            if ptr.is_null() {
                return 1;
            }
            if !(*source).buffer.is_null() {
                for y in 0..src_h {
                    let s_offset = y * src_pitch;
                    let d_offset = y * target_pitch;
                    let copy_w = src_w.min(src_pitch);
                    ptr::copy_nonoverlapping((*source).buffer.add(s_offset), ptr.add(d_offset), copy_w);
                }
            }
            *target = FT_Bitmap {
                rows: (*source).rows,
                width: (*source).width,
                pitch: target_pitch as c_int,
                buffer: ptr,
                num_grays: (*source).num_grays,
                pixel_mode: FT_PIXEL_MODE_GRAY,
                palette_mode: 0,
                palette: ptr::null_mut(),
            };
            0
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn FT_Bitmap_Copy(
    library: *mut c_void,
    source: *const FT_Bitmap,
    target: *mut FT_Bitmap,
) -> c_int {
    FT_Bitmap_Convert(library, source, target, 1)
}

#[no_mangle]
pub unsafe extern "C" fn FT_GlyphSlot_Embolden(slot: *mut FT_GlyphSlotRec) {
    if slot.is_null() {
        return;
    }
    (*slot).advance.x += 1 << 6;
}

#[no_mangle]
pub unsafe extern "C" fn FT_GlyphSlot_Oblique(slot: *mut FT_GlyphSlotRec) {
    if slot.is_null() {
        return;
    }
    let matrix = FT_Matrix {
        xx: 0x10000,
        xy: 0x0366A,
        yx: 0,
        yy: 0x10000,
    };
    FT_Outline_Transform(&mut (*slot).outline, &matrix);
}

// =========================================================================
// Multiple Masters & Variable Fonts Stubs
// =========================================================================

#[no_mangle]
pub unsafe extern "C" fn FT_Get_MM_Var(
    _face: *mut FT_FaceRec,
    amaster: *mut *mut c_void,
) -> c_int {
    if !amaster.is_null() {
        *amaster = ptr::null_mut();
    }
    0x8F
}

#[no_mangle]
pub unsafe extern "C" fn FT_Done_MM_Var(
    _library: *mut c_void,
    _master: *mut c_void,
) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Set_Var_Design_Coordinates(
    _face: *mut FT_FaceRec,
    _num_coords: c_uint,
    _coords: *const c_long,
) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Var_Design_Coordinates(
    _face: *mut FT_FaceRec,
    _num_coords: c_uint,
    _coords: *mut c_long,
) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Var_Blend_Coordinates(
    _face: *mut FT_FaceRec,
    _num_coords: c_uint,
    _coords: *mut c_short,
) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Set_Named_Instance(
    _face: *mut FT_FaceRec,
    _instance_index: c_uint,
) -> c_int {
    0
}

// =========================================================================
// Color Glyphs Fallback Stubs (COLR / CPAL)
// =========================================================================

#[no_mangle]
pub unsafe extern "C" fn FT_Palette_Data_Get(_face: *mut FT_FaceRec, _apalette: *mut c_void) -> c_int {
    1
}

#[no_mangle]
pub unsafe extern "C" fn FT_Palette_Select(
    _face: *mut FT_FaceRec,
    _palette_index: c_ushort,
    _apalette: *mut *mut c_void,
) -> c_int {
    1
}

#[no_mangle]
pub unsafe extern "C" fn FT_Palette_Set_Foreground_Color(
    _face: *mut FT_FaceRec,
    _color: c_ulong,
) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Color_Glyph_ClipBox(
    _face: *mut FT_FaceRec,
    _base_glyph: c_uint,
    _clip_box: *mut c_void,
) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Color_Glyph_Layer(
    _face: *mut FT_FaceRec,
    _base_glyph: c_uint,
    _glyph_index: *mut c_uint,
    _color_index: *mut c_uint,
    _iterator: *mut c_void,
) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Color_Glyph_Paint(
    _face: *mut FT_FaceRec,
    _base_glyph: c_uint,
    _root_transform: c_int,
    _paint: *mut c_void,
) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Colorline_Stops(
    _face: *mut FT_FaceRec,
    _color_stop: *mut c_void,
    _iterator: *mut c_void,
) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Paint(
    _face: *mut FT_FaceRec,
    _opaque_paint: *mut c_void,
    _paint: *mut c_void,
) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Paint_Layers(
    _face: *mut FT_FaceRec,
    _iterator: *mut c_void,
    _paint: *mut c_void,
) -> c_int {
    0
}

// =========================================================================
// Fontconfig, Cairo and HarfBuzz Required Symbols
// =========================================================================

#[repr(C)]
pub struct PS_FontInfoRec {
    pub version: *mut c_char,
    pub notice: *mut c_char,
    pub full_name: *mut c_char,
    pub family_name: *mut c_char,
    pub weight: *mut c_char,
    pub italic_angle: c_long,
    pub is_fixed_pitch: c_uchar,
    pub underline_position: c_short,
    pub underline_thickness: c_ushort,
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_PS_Font_Info(
    face: *mut FT_FaceRec,
    afont_info: *mut PS_FontInfoRec,
) -> c_int {
    if face.is_null() || afont_info.is_null() {
        return 1;
    }
    (*afont_info).version = ptr::null_mut();
    (*afont_info).notice = ptr::null_mut();
    (*afont_info).full_name = (*face).family_name;
    (*afont_info).family_name = (*face).family_name;
    (*afont_info).weight = (*face).style_name;
    (*afont_info).italic_angle = 0;
    (*afont_info).is_fixed_pitch = 0;
    (*afont_info).underline_position = (*face).underline_position;
    (*afont_info).underline_thickness = (*face).underline_thickness as c_ushort;
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_PS_Font_Private(
    _face: *mut FT_FaceRec,
    _afont_private: *mut c_void,
) -> c_int {
    6 // FT_Err_Invalid_Argument
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_PS_Font_Value(
    _face: *mut FT_FaceRec,
    _key: c_uint,
    _idx: c_uint,
    _value: *mut c_void,
    _value_len: c_long,
) -> c_long {
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Has_PS_Glyph_Names(face: *mut FT_FaceRec) -> c_int {
    if face.is_null() {
        return 0;
    }
    1
}

#[repr(C)]
pub struct BDF_PropertyRec {
    pub prop_type: c_int,
    pub u: [usize; 2],
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_BDF_Property(
    _face: *mut FT_FaceRec,
    _prop_name: *const c_char,
    _aproperty: *mut BDF_PropertyRec,
) -> c_int {
    6 // FT_Err_Invalid_Argument (not a BDF font)
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_BDF_Charset_ID(
    _face: *mut FT_FaceRec,
    _acharset_encoding: *mut *const c_char,
    _acharset_registry: *mut *const c_char,
) -> c_int {
    6 // FT_Err_Invalid_Argument
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_FSType_Flags(_face: *mut FT_FaceRec) -> c_ushort {
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Gasp(_face: *mut FT_FaceRec, _ppem: c_uint) -> c_int {
    0 // FT_GASP_NO_TABLE
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_WinFNT_Header(
    _face: *mut FT_FaceRec,
    _aheader: *mut c_void,
) -> c_int {
    6 // FT_Err_Invalid_Argument
}

#[no_mangle]
pub unsafe extern "C" fn FT_Face_GetCharVariantIndex(
    face: *mut FT_FaceRec,
    charcode: c_ulong,
    _variantSelector: c_ulong,
) -> c_uint {
    FT_Get_Char_Index(face, charcode)
}

#[no_mangle]
pub unsafe extern "C" fn FT_Face_GetCharVariantIsDefault(
    _face: *mut FT_FaceRec,
    _charcode: c_ulong,
    _variantSelector: c_ulong,
) -> c_int {
    1
}

#[no_mangle]
pub unsafe extern "C" fn FT_Face_GetVariantSelectors(_face: *mut FT_FaceRec) -> *mut c_uint {
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn FT_Face_GetVariantsOfChar(
    _face: *mut FT_FaceRec,
    _charcode: c_ulong,
) -> *mut c_uint {
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn FT_Face_GetCharsOfVariant(
    _face: *mut FT_FaceRec,
    _variantSelector: c_ulong,
) -> *mut c_uint {
    ptr::null_mut()
}


#[no_mangle]
pub unsafe extern "C" fn FT_Get_Track_Kerning(
    _face: *mut FT_FaceRec,
    _point_size: c_long,
    _degree: c_int,
    akerning: *mut c_long,
) -> c_int {
    if !akerning.is_null() {
        *akerning = 0;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Var_Axis_Flags(
    _master: *mut c_void,
    _axis_index: c_uint,
    flags: *mut c_uint,
) -> c_int {
    if !flags.is_null() {
        *flags = 0;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Get_Default_Named_Instance(
    _face: *mut FT_FaceRec,
    instance_index: *mut c_uint,
) -> c_int {
    if !instance_index.is_null() {
        *instance_index = 0;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Face_CheckTrueTypePatents(_face: *mut FT_FaceRec) -> c_uchar {
    0
}

#[no_mangle]
pub unsafe extern "C" fn FT_Face_SetUnpatentedHinting(
    _face: *mut FT_FaceRec,
    _value: c_uchar,
) -> c_uchar {
    0
}

#[no_mangle]
pub extern "C" fn FT_MulDiv(a: c_long, b: c_long, c: c_long) -> c_long {
    if c == 0 {
        return 0;
    }
    ((a as i128 * b as i128) / c as i128) as c_long
}

#[no_mangle]
pub extern "C" fn FT_DivFix(a: c_long, b: c_long) -> c_long {
    if b == 0 {
        return 0;
    }
    (((a as i128) << 16) / b as i128) as c_long
}

#[no_mangle]
pub unsafe extern "C" fn FT_Matrix_Multiply(a: *const FT_Matrix, b: *mut FT_Matrix) {
    if a.is_null() || b.is_null() {
        return;
    }
    let (xx1, xy1, yx1, yy1) = ((*a).xx, (*a).xy, (*a).yx, (*a).yy);
    let (xx2, xy2, yx2, yy2) = ((*b).xx, (*b).xy, (*b).yx, (*b).yy);

    (*b).xx = (FT_MulFix(xx1, xx2) + FT_MulFix(xy1, yx2)) as c_long;
    (*b).xy = (FT_MulFix(xx1, xy2) + FT_MulFix(xy1, yy2)) as c_long;
    (*b).yx = (FT_MulFix(yx1, xx2) + FT_MulFix(yy1, yx2)) as c_long;
    (*b).yy = (FT_MulFix(yx1, xy2) + FT_MulFix(yy1, yy2)) as c_long;
}

#[no_mangle]
pub unsafe extern "C" fn FT_Matrix_Invert(matrix: *mut FT_Matrix) -> c_int {
    if matrix.is_null() {
        return 1;
    }
    let (xx, xy, yx, yy) = ((*matrix).xx, (*matrix).xy, (*matrix).yx, (*matrix).yy);
    let det = FT_MulFix(xx, yy) - FT_MulFix(xy, yx);
    if det == 0 {
        return 1;
    }
    (*matrix).xx = FT_DivFix(yy, det);
    (*matrix).xy = -FT_DivFix(xy, det);
    (*matrix).yx = -FT_DivFix(yx, det);
    (*matrix).yy = FT_DivFix(xx, det);
    0
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ft_new_face_query_count() {
        let mut lib: *mut c_void = ptr::null_mut();
        assert_eq!(FT_Init_FreeType(&mut lib), 0);

        let font_path = if std::path::Path::new("/usr/share/fonts/google-noto-vf/NotoSans[wght].ttf").exists() {
            "/usr/share/fonts/google-noto-vf/NotoSans[wght].ttf"
        } else if std::path::Path::new("/usr/share/fonts/adwaita-sans-fonts/AdwaitaSans-Regular.ttf").exists() {
            "/usr/share/fonts/adwaita-sans-fonts/AdwaitaSans-Regular.ttf"
        } else if std::path::Path::new("/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf").exists() {
            "/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf"
        } else if std::path::Path::new("/System/Library/Fonts/Supplemental/Arial.ttf").exists() {
            "/System/Library/Fonts/Supplemental/Arial.ttf"
        } else if std::path::Path::new("/Library/Fonts/Arial.ttf").exists() {
            "/Library/Fonts/Arial.ttf"
        } else {
            "/System/Library/Fonts/Menlo.ttc"
        };

        let c_path = CString::new(font_path).unwrap();
        let mut face: *mut FT_FaceRec = ptr::null_mut();

        // Query mode: face_index = -1 (used by Fontconfig FcFreeTypeQueryFace)
        let err = FT_New_Face(lib, c_path.as_ptr(), -1, &mut face);
        assert_eq!(err, 0);
        assert!(!face.is_null());
        unsafe {
            assert!((*face).num_faces >= 1);
            assert_eq!((*face).face_index, -1);
            assert_eq!(FT_Done_Face(face), 0);
        }

        // Standard mode: face_index = 0
        let mut face0: *mut FT_FaceRec = ptr::null_mut();
        let err0 = FT_New_Face(lib, c_path.as_ptr(), 0, &mut face0);
        assert_eq!(err0, 0);
        assert!(!face0.is_null());
        unsafe {
            assert!((*face0).num_faces >= 1);
            assert_eq!((*face0).face_index, 0);

            // Test variable font named instance (upper 16 bits set)
            let mut face_inst: *mut FT_FaceRec = ptr::null_mut();
            let err_inst = FT_New_Face(lib, c_path.as_ptr(), 1 << 16, &mut face_inst);
            assert_eq!(err_inst, 0);
            assert!(!face_inst.is_null());

            // Test loading and rendering a regular character
            let gid_a = FT_Get_Char_Index(face_inst, 'A' as c_ulong);
            assert!(gid_a > 0);
            assert_eq!(FT_Load_Glyph(face_inst, gid_a, FT_LOAD_DEFAULT), 0);
            let slot_a = (*face_inst).glyph;
            assert_eq!(FT_Render_Glyph(slot_a, 0), 0);
            assert_eq!((*slot_a).format, FT_GLYPH_FORMAT_BITMAP);
            assert_eq!((*slot_a).bitmap.pitch % 4, 0); // 4-byte aligned stride

            // Verify metrics synchronization and baseline consistency
            assert_eq!((*slot_a).metrics.horiBearingX >> 6, (*slot_a).bitmap_left as c_long);
            assert_eq!((*slot_a).metrics.horiBearingY >> 6, (*slot_a).bitmap_top as c_long);
            assert_eq!((*slot_a).metrics.width >> 6, (*slot_a).bitmap.width as c_long);
            assert_eq!((*slot_a).metrics.height >> 6, (*slot_a).bitmap.rows as c_long);

            // Test whitespace (empty glyph) handling
            let gid_space = FT_Get_Char_Index(face_inst, ' ' as c_ulong);
            assert_eq!(FT_Load_Glyph(face_inst, gid_space, FT_LOAD_DEFAULT), 0);
            let slot_sp = (*face_inst).glyph;
            assert_eq!(FT_Render_Glyph(slot_sp, 0), 0);
            assert_eq!((*slot_sp).format, FT_GLYPH_FORMAT_BITMAP);
            assert_eq!((*slot_sp).bitmap.width, 0);
            assert_eq!((*slot_sp).bitmap.rows, 0);
            assert!((*slot_sp).advance.x > 0); // Advance width must be preserved!

            // Test descender glyph ('g') baseline metric consistency
            let gid_g = FT_Get_Char_Index(face_inst, 'g' as c_ulong);
            if gid_g > 0 {
                assert_eq!(FT_Load_Glyph(face_inst, gid_g, FT_LOAD_DEFAULT), 0);
                let slot_g = (*face_inst).glyph;
                assert_eq!(FT_Render_Glyph(slot_g, 0), 0);
                assert_eq!((*slot_g).metrics.horiBearingX >> 6, (*slot_g).bitmap_left as c_long);
                assert_eq!((*slot_g).metrics.horiBearingY >> 6, (*slot_g).bitmap_top as c_long);
            }

            // Test FT_Get_Glyph and FT_Glyph_To_Bitmap
            assert_eq!(FT_Load_Glyph(face_inst, gid_a, FT_LOAD_DEFAULT), 0);
            let mut glyph: *mut FT_GlyphRec = ptr::null_mut();
            assert_eq!(FT_Get_Glyph(slot_a, &mut glyph), 0);
            assert!(!glyph.is_null());
            assert_eq!((*glyph).format, FT_GLYPH_FORMAT_OUTLINE);

            assert_eq!(FT_Glyph_To_Bitmap(&mut glyph, 0, ptr::null_mut(), 1), 0);
            assert!(!glyph.is_null());
            assert_eq!((*glyph).format, FT_GLYPH_FORMAT_BITMAP);
            let bg = glyph as *mut FT_BitmapGlyphRec;
            assert!((*bg).bitmap.width > 0);
            assert!((*bg).bitmap.rows > 0);
            FT_Done_Glyph(glyph);

            assert_eq!(FT_Done_Face(face_inst), 0);
            assert_eq!(FT_Done_Face(face0), 0);
        }

        assert_eq!(FT_Done_FreeType(lib), 0);
    }

    #[test]
    fn test_bitmap_mono_convert() {
        // 8x2 1-bit monochrome bitmap
        let mono_data: [u8; 2] = [0b10101010, 0b11001100];
        let src = FT_Bitmap {
            rows: 2,
            width: 8,
            pitch: 1,
            buffer: mono_data.as_ptr() as *mut c_uchar,
            num_grays: 2,
            pixel_mode: FT_PIXEL_MODE_MONO,
            palette_mode: 0,
            palette: ptr::null_mut(),
        };

        let mut dst = FT_Bitmap {
            rows: 0,
            width: 0,
            pitch: 0,
            buffer: ptr::null_mut(),
            num_grays: 0,
            pixel_mode: 0,
            palette_mode: 0,
            palette: ptr::null_mut(),
        };

        unsafe {
            assert_eq!(FT_Bitmap_Convert(ptr::null_mut(), &src, &mut dst, 4), 0);
            assert_eq!(dst.pixel_mode, FT_PIXEL_MODE_GRAY);
            assert_eq!(dst.num_grays, 256);
            assert_eq!(dst.width, 8);
            assert_eq!(dst.rows, 2);
            assert_eq!(dst.pitch % 4, 0);

            let row0 = slice::from_raw_parts(dst.buffer, 8);
            assert_eq!(row0[0], 255);
            assert_eq!(row0[1], 0);
            assert_eq!(row0[2], 255);
            assert_eq!(row0[3], 0);
            assert_eq!(row0[4], 255);
            assert_eq!(row0[5], 0);
            assert_eq!(row0[6], 255);
            assert_eq!(row0[7], 0);

            assert_eq!(FT_Bitmap_Done(ptr::null_mut(), &mut dst), 0);
        }
    }

    #[test]
    fn test_ft_open_stream() {
        let mut lib: *mut c_void = ptr::null_mut();
        assert_eq!(FT_Init_FreeType(&mut lib), 0);

        let font_path = if std::path::Path::new("/usr/share/fonts/adwaita-sans-fonts/AdwaitaSans-Regular.ttf").exists() {
            "/usr/share/fonts/adwaita-sans-fonts/AdwaitaSans-Regular.ttf"
        } else if std::path::Path::new("/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf").exists() {
            "/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf"
        } else if std::path::Path::new("/System/Library/Fonts/Helvetica.ttc").exists() {
            "/System/Library/Fonts/Helvetica.ttc"
        } else {
            "/System/Library/Fonts/Menlo.ttc"
        };

        let font_bytes = std::fs::read(font_path).expect("Failed to read font file");
        let mut mem_stream = FT_StreamRec {
            base: font_bytes.as_ptr() as *mut c_uchar,
            size: font_bytes.len() as c_ulong,
            pos: 0,
            descriptor: ptr::null_mut(),
            pathname: ptr::null_mut(),
            read: None,
            close: None,
            memory: ptr::null_mut(),
            cursor: ptr::null_mut(),
            limit: ptr::null_mut(),
        };

        let open_args = FT_Open_Args {
            flags: FT_OPEN_STREAM,
            memory_base: ptr::null(),
            memory_size: 0,
            pathname: ptr::null_mut(),
            stream: &mut mem_stream,
            driver: ptr::null_mut(),
            num_params: 0,
            params: ptr::null_mut(),
        };

        let mut face: *mut FT_FaceRec = ptr::null_mut();
        unsafe {
            let res = FT_Open_Face(lib, &open_args, 0, &mut face);
            assert_eq!(res, 0);
            assert!(!face.is_null());
            assert!((*face).num_glyphs > 0);

            // Test FT_Attach stubs
            assert_eq!(FT_Attach_File(face, ptr::null()), 0);
            let mut attach_args = open_args;
            assert_eq!(FT_Attach_Stream(face, &mut attach_args), 0);

            assert_eq!(FT_Done_Face(face), 0);
            assert_eq!(FT_Done_FreeType(lib), 0);
        }
    }
}
