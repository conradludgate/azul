//! Provides a public API with datatypes used to describe style properties of DOM nodes.

use allsorts::tables::os2::Os2;
use allsorts::tables::{HeadTable, HheaTable};

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

pub struct FontMetrics {
    pub head: HeadTable,
    pub hhea: HheaTable,
    pub os2: Os2,
}

impl FontMetrics {
    /// If set, use `OS/2.sTypoAscender - OS/2.sTypoDescender + OS/2.sTypoLineGap` to calculate the height
    ///
    /// See [`USE_TYPO_METRICS`](https://docs.microsoft.com/en-us/typography/opentype/spec/os2#fss)
    pub fn use_typo_metrics(&self) -> bool {
        self.os2.fs_selection & (1 << 7) != 0
    }

    pub fn get_ascender_unscaled(&self) -> i16 {
        let use_typo = if !self.use_typo_metrics() {
            None
        } else {
            self.os2.version0.as_ref().map(|x| x.s_typo_ascender)
        };
        match use_typo {
            Some(s) => s,
            None => self.hhea.ascender,
        }
    }

    /// NOTE: descender is NEGATIVE
    pub fn get_descender_unscaled(&self) -> i16 {
        let use_typo = if !self.use_typo_metrics() {
            None
        } else {
            self.os2.version0.as_ref().map(|x| x.s_typo_descender)
        };
        match use_typo {
            Some(s) => s,
            None => self.hhea.descender,
        }
    }

    pub fn get_line_gap_unscaled(&self) -> i16 {
        let use_typo = if !self.use_typo_metrics() {
            None
        } else {
            self.os2.version0.as_ref().map(|x| x.s_typo_line_gap)
        };
        match use_typo {
            Some(s) => s,
            None => self.hhea.line_gap,
        }
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
