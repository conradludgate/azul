//! General crate for text layout / text shaping
//!
//! ![Text layout functions and ](https://i.imgur.com/1T7a1VR.png)
//!
//! # Example
//!
//! ```rust,ignore,no_run
//! use azul_text_layout::{
//!     text_layout::{split_text_into_words, words_to_scaled_words},
//!     text_shaping::get_font_metrics_freetype,
//! };
//!
//! let text = "hello";
//! let font_size = 14.0; // px
//! let font = include_bytes!("Helvetica.ttf");
//! let font_index = 0; // only for fonts with font collections
//! let font_metrics = get_font_metrics_freetype(&font, font_index);
//! let words = split_text_into_words(text);
//! let scaled_words = words_to_scaled_words(&words, &font, font_index as u32, font_metrics, font_size);
//!
//! let total_width = scaled_words.items.iter().map(|i| i.word_width).sum();
//! ```
//!
//! # Full text layout
//!
//! ```rust,ignore,no_run
//! use azul_text_layout::{text_layout, text_shaping::get_font_metrics_freetype};
//! use azul_css::{LayoutSize, StyleTextAlignmentHorz};
//! use azul_core::ui_solver::ResolvedTextLayoutOptions;
//!
//! // set all options of the text
//! let text = "hello";
//! let font_size = 14.0; // px
//! let font_bytes = include_bytes!("Helvetica.ttf");
//! let font_index = 0; // only for fonts with font collections
//! let text_layout_options = ResolvedTextLayoutOptions {
//!     font_size_px: font_size,
//!     line_height: None,
//!     letter_spacing: None,
//!     word_spacing: None,
//!     tab_width: None,
//!     // for line breaking, maximum width that a line can have
//!     max_horizontal_width: Some(400.0), // px
//!     leading: None,
//!     holes: Vec::new(),
//! };
//!
//! // Cache the font metrics of the given font (baseline, height, etc.)
//! let font_metrics = get_font_metrics_freetype(font_bytes, font_index as i32);
//! // "Hello World" => ["Hello", "World"]
//! let words = text_layout::split_text_into_words(text);
//! // "Hello" @ 14px => Size { width: 50px, height: 14px }
//! let scaled_words = text_layout::words_to_scaled_words(&words, font_bytes, font_index, font_metrics, text_layout_options.font_size_px);
//! // Calculate the origin of the word relative to the line
//! let word_positions = text_layout::position_words(&words, &scaled_words, &text_layout_options);
//! // Calculate the origin of the line relative to (0, 0)
//! let mut inline_text_layout = text_layout::word_positions_to_inline_text_layout(&word_positions, &scaled_words);
//! // Align the line horizontally
//! inline_text_layout.align_children_horizontal(StyleTextAlignmentHorz::Center);
//! // Calculate the glyph positons (line_offset + word_offset + glyph_offset)
//! let layouted_glyphs = text_layout::get_layouted_glyphs(&word_positions, &scaled_words, &inline_text_layout);
//!
//! println!("{:#?}", inline_text_layout); // get infos about word offset, line breaking, etc.
//! println!("{:#?}", layouted_glyphs); // get the final glyph positions relative to the origin
//! ```

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/maps4print/azul/master/assets/images/azul_logo_full_min.svg.png",
    html_favicon_url = "https://raw.githubusercontent.com/maps4print/azul/master/assets/images/favicon.ico"
)]

pub mod css;
pub mod css_parser;
pub mod logical;
pub mod ui_solver;
mod words;

#[macro_use]
extern crate tinyvec;

use self::css::{FontData, FontRef};
use crate::text_shaping::ParsedFont;
use core::ffi::c_void;

pub mod script;
pub mod text_layout;
pub mod text_shaping;

use css::FontMetrics;

fn parsed_font_destructor(ptr: *mut c_void) {
    unsafe {
        let _ = Box::from_raw(ptr as *mut ParsedFont);
    }
}

pub fn parse_font_fn(source: LoadedFontSource) -> Option<FontRef> {
    crate::text_layout::parse_font(
        source.data.as_ref(),
        source.index as usize,
        source.load_outlines,
    )
    .map(|parsed_font| {
        FontRef::new(FontData {
            bytes: source.data,
            font_index: source.index,
            parsed: Box::into_raw(Box::new(parsed_font)) as *const c_void,
            parsed_destructor: parsed_font_destructor,
        })
    })
}

pub fn get_font_metrics_fontref(font_ref: &FontRef) -> FontMetrics {
    let parsed_font = unsafe { &*(font_ref.get_data().parsed as *const ParsedFont) };
    parsed_font.font_metrics
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoadedFontSource {
    pub data: Vec<u8>,
    pub index: u32,
    pub load_outlines: bool,
}
