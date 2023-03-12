//! Provides a public API with datatypes used to describe style properties of DOM nodes.

use crate::text_shaping::ParsedFont;
use core::fmt;
use core::hash::Hash;

/// Horizontal text alignment enum (left, center, right) - default: `Center`
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum StyleTextAlign {
    Left,
    Center,
    Right,
}

/// Vertical text alignment enum (top, center, bottom) - default: `Center`
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum StyleVerticalAlign {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct FontMetrics {
    // head table
    pub units_per_em: u16,
    pub font_flags: u16,
    pub x_min: i16,
    pub y_min: i16,
    pub x_max: i16,
    pub y_max: i16,

    // hhea table
    pub ascender: i16,
    pub descender: i16,
    pub line_gap: i16,
    pub advance_width_max: u16,
    pub min_left_side_bearing: i16,
    pub min_right_side_bearing: i16,
    pub x_max_extent: i16,
    pub caret_slope_rise: i16,
    pub caret_slope_run: i16,
    pub caret_offset: i16,
    pub num_h_metrics: u16,

    // os/2 table
    pub x_avg_char_width: i16,
    pub us_weight_class: u16,
    pub us_width_class: u16,
    pub fs_type: u16,
    pub y_subscript_x_size: i16,
    pub y_subscript_y_size: i16,
    pub y_subscript_x_offset: i16,
    pub y_subscript_y_offset: i16,
    pub y_superscript_x_size: i16,
    pub y_superscript_y_size: i16,
    pub y_superscript_x_offset: i16,
    pub y_superscript_y_offset: i16,
    pub y_strikeout_size: i16,
    pub y_strikeout_position: i16,
    pub s_family_class: i16,
    pub panose: [u8; 10],
    pub ul_unicode_range1: u32,
    pub ul_unicode_range2: u32,
    pub ul_unicode_range3: u32,
    pub ul_unicode_range4: u32,
    pub ach_vend_id: u32,
    pub fs_selection: u16,
    pub us_first_char_index: u16,
    pub us_last_char_index: u16,

    // os/2 version 0 table
    pub s_typo_ascender: Option<i16>,
    pub s_typo_descender: Option<i16>,
    pub s_typo_line_gap: Option<i16>,
    pub us_win_ascent: Option<u16>,
    pub us_win_descent: Option<u16>,

    // os/2 version 1 table
    pub ul_code_page_range1: Option<u32>,
    pub ul_code_page_range2: Option<u32>,

    // os/2 version 2 table
    pub sx_height: Option<i16>,
    pub s_cap_height: Option<i16>,
    pub us_default_char: Option<u16>,
    pub us_break_char: Option<u16>,
    pub us_max_context: Option<u16>,

    // os/2 version 3 table
    pub us_lower_optical_point_size: Option<u16>,
    pub us_upper_optical_point_size: Option<u16>,
}

impl FontMetrics {
    /// If set, use `OS/2.sTypoAscender - OS/2.sTypoDescender + OS/2.sTypoLineGap` to calculate the height
    ///
    /// See [`USE_TYPO_METRICS`](https://docs.microsoft.com/en-us/typography/opentype/spec/os2#fss)
    pub fn use_typo_metrics(&self) -> bool {
        self.fs_selection & (1 << 7) != 0
    }

    pub fn get_ascender_unscaled(&self) -> i16 {
        let use_typo = if !self.use_typo_metrics() {
            None
        } else {
            self.s_typo_ascender
        };
        match use_typo {
            Some(s) => s,
            None => self.ascender,
        }
    }

    /// NOTE: descender is NEGATIVE
    pub fn get_descender_unscaled(&self) -> i16 {
        let use_typo = if !self.use_typo_metrics() {
            None
        } else {
            self.s_typo_descender
        };
        match use_typo {
            Some(s) => s,
            None => self.descender,
        }
    }

    pub fn get_line_gap_unscaled(&self) -> i16 {
        let use_typo = if !self.use_typo_metrics() {
            None
        } else {
            self.s_typo_line_gap
        };
        match use_typo {
            Some(s) => s,
            None => self.line_gap,
        }
    }

    pub fn get_ascender(&self, target_font_size: f32) -> f32 {
        self.get_ascender_unscaled() as f32 / self.units_per_em as f32 * target_font_size
    }
    pub fn get_descender(&self, target_font_size: f32) -> f32 {
        self.get_descender_unscaled() as f32 / self.units_per_em as f32 * target_font_size
    }
    pub fn get_line_gap(&self, target_font_size: f32) -> f32 {
        self.get_line_gap_unscaled() as f32 / self.units_per_em as f32 * target_font_size
    }

    pub fn get_x_min(&self, target_font_size: f32) -> f32 {
        self.x_min as f32 / self.units_per_em as f32 * target_font_size
    }
    pub fn get_y_min(&self, target_font_size: f32) -> f32 {
        self.y_min as f32 / self.units_per_em as f32 * target_font_size
    }
    pub fn get_x_max(&self, target_font_size: f32) -> f32 {
        self.x_max as f32 / self.units_per_em as f32 * target_font_size
    }
    pub fn get_y_max(&self, target_font_size: f32) -> f32 {
        self.y_max as f32 / self.units_per_em as f32 * target_font_size
    }
    pub fn get_advance_width_max(&self, target_font_size: f32) -> f32 {
        self.advance_width_max as f32 / self.units_per_em as f32 * target_font_size
    }
    pub fn get_min_left_side_bearing(&self, target_font_size: f32) -> f32 {
        self.min_left_side_bearing as f32 / self.units_per_em as f32 * target_font_size
    }
    pub fn get_min_right_side_bearing(&self, target_font_size: f32) -> f32 {
        self.min_right_side_bearing as f32 / self.units_per_em as f32 * target_font_size
    }
    pub fn get_x_max_extent(&self, target_font_size: f32) -> f32 {
        self.x_max_extent as f32 / self.units_per_em as f32 * target_font_size
    }
    pub fn get_x_avg_char_width(&self, target_font_size: f32) -> f32 {
        self.x_avg_char_width as f32 / self.units_per_em as f32 * target_font_size
    }
    pub fn get_y_subscript_x_size(&self, target_font_size: f32) -> f32 {
        self.y_subscript_x_size as f32 / self.units_per_em as f32 * target_font_size
    }
    pub fn get_y_subscript_y_size(&self, target_font_size: f32) -> f32 {
        self.y_subscript_y_size as f32 / self.units_per_em as f32 * target_font_size
    }
    pub fn get_y_subscript_x_offset(&self, target_font_size: f32) -> f32 {
        self.y_subscript_x_offset as f32 / self.units_per_em as f32 * target_font_size
    }
    pub fn get_y_subscript_y_offset(&self, target_font_size: f32) -> f32 {
        self.y_subscript_y_offset as f32 / self.units_per_em as f32 * target_font_size
    }
    pub fn get_y_superscript_x_size(&self, target_font_size: f32) -> f32 {
        self.y_superscript_x_size as f32 / self.units_per_em as f32 * target_font_size
    }
    pub fn get_y_superscript_y_size(&self, target_font_size: f32) -> f32 {
        self.y_superscript_y_size as f32 / self.units_per_em as f32 * target_font_size
    }
    pub fn get_y_superscript_x_offset(&self, target_font_size: f32) -> f32 {
        self.y_superscript_x_offset as f32 / self.units_per_em as f32 * target_font_size
    }
    pub fn get_y_superscript_y_offset(&self, target_font_size: f32) -> f32 {
        self.y_superscript_y_offset as f32 / self.units_per_em as f32 * target_font_size
    }
    pub fn get_y_strikeout_size(&self, target_font_size: f32) -> f32 {
        self.y_strikeout_size as f32 / self.units_per_em as f32 * target_font_size
    }
    pub fn get_y_strikeout_position(&self, target_font_size: f32) -> f32 {
        self.y_strikeout_position as f32 / self.units_per_em as f32 * target_font_size
    }

    pub fn get_s_typo_ascender(&self, target_font_size: f32) -> Option<f32> {
        self.s_typo_ascender
            .map(|s| s as f32 / self.units_per_em as f32 * target_font_size)
    }
    pub fn get_s_typo_descender(&self, target_font_size: f32) -> Option<f32> {
        self.s_typo_descender
            .map(|s| s as f32 / self.units_per_em as f32 * target_font_size)
    }
    pub fn get_s_typo_line_gap(&self, target_font_size: f32) -> Option<f32> {
        self.s_typo_line_gap
            .map(|s| s as f32 / self.units_per_em as f32 * target_font_size)
    }
    pub fn get_us_win_ascent(&self, target_font_size: f32) -> Option<f32> {
        self.us_win_ascent
            .map(|s| s as f32 / self.units_per_em as f32 * target_font_size)
    }
    pub fn get_us_win_descent(&self, target_font_size: f32) -> Option<f32> {
        self.us_win_descent
            .map(|s| s as f32 / self.units_per_em as f32 * target_font_size)
    }
    pub fn get_sx_height(&self, target_font_size: f32) -> Option<f32> {
        self.sx_height
            .map(|s| s as f32 / self.units_per_em as f32 * target_font_size)
    }
    pub fn get_s_cap_height(&self, target_font_size: f32) -> Option<f32> {
        self.s_cap_height
            .map(|s| s as f32 / self.units_per_em as f32 * target_font_size)
    }
}

pub struct FontData {
    // T = ParsedFont
    /// Bytes of the font file, either &'static (never changing bytes) or a Vec<u8>.
    pub bytes: Vec<u8>,
    /// Index of the font in the file (if not known, set to 0) -
    /// only relevant if the file is a font collection
    pub font_index: u32,
    pub parsed: ParsedFont,
}

impl fmt::Debug for FontData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FontData")
            .field("bytes", &self.bytes)
            .field("font_index", &self.font_index)
            .finish()
    }
}
