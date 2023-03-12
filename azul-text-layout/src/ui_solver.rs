use crate::{
    css::css_properties::{StyleTextAlign, StyleVerticalAlign},
    logical::{LogicalRect, LogicalSize},
};

pub const DEFAULT_LINE_HEIGHT: f32 = 1.0;
pub const DEFAULT_WORD_SPACING: f32 = 1.0;
pub const DEFAULT_TAB_WIDTH: f32 = 4.0;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[repr(C)]
pub struct InlineTextLayout {
    pub lines: Vec<InlineTextLine>,
    pub content_size: LogicalSize,
}

/// NOTE: The bounds of the text line is the TOP left corner (relative to the text origin),
/// but the word_position is the BOTTOM left corner (relative to the text line)
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[repr(C)]
pub struct InlineTextLine {
    pub bounds: LogicalRect,
    /// At which word does this line start?
    pub word_start: usize,
    /// At which word does this line end
    pub word_end: usize,
}

impl InlineTextLine {
    pub const fn new(bounds: LogicalRect, word_start: usize, word_end: usize) -> Self {
        Self {
            bounds,
            word_start,
            word_end,
        }
    }
}

impl InlineTextLayout {
    #[inline]
    pub fn get_leading(&self) -> f32 {
        match self.lines.first() {
            None => 0.0,
            Some(s) => s.bounds.origin.x,
        }
    }

    #[inline]
    pub fn get_trailing(&self) -> f32 {
        match self.lines.first() {
            None => 0.0,
            Some(s) => s.bounds.origin.x + s.bounds.size.width,
        }
    }

    /// Align the lines horizontal to *their bounding box*
    pub fn align_children_horizontal(
        &mut self,
        parent_size: &LogicalSize,
        horizontal_alignment: StyleTextAlign,
    ) {
        let shift_multiplier = match calculate_horizontal_shift_multiplier(horizontal_alignment) {
            None => return,
            Some(s) => s,
        };

        for line in self.lines.iter_mut() {
            line.bounds.origin.x += shift_multiplier * (parent_size.width - line.bounds.size.width);
        }
    }

    /// Align the lines vertical to *their parents container*
    pub fn align_children_vertical_in_parent_bounds(
        &mut self,
        parent_size: &LogicalSize,
        vertical_alignment: StyleVerticalAlign,
    ) {
        let shift_multiplier = match calculate_vertical_shift_multiplier(vertical_alignment) {
            None => return,
            Some(s) => s,
        };

        let glyphs_vertical_bottom = self.lines.last().map(|l| l.bounds.origin.y).unwrap_or(0.0);
        let vertical_shift = (parent_size.height - glyphs_vertical_bottom) * shift_multiplier;

        for line in self.lines.iter_mut() {
            line.bounds.origin.y += vertical_shift;
        }
    }
}

#[inline]
pub fn calculate_horizontal_shift_multiplier(horizontal_alignment: StyleTextAlign) -> Option<f32> {
    use crate::css::css_properties::StyleTextAlign::*;
    match horizontal_alignment {
        Left => None,
        Center => Some(0.5), // move the line by the half width
        Right => Some(1.0),  // move the line by the full width
    }
}

#[inline]
pub fn calculate_vertical_shift_multiplier(vertical_alignment: StyleVerticalAlign) -> Option<f32> {
    use crate::css::css_properties::StyleVerticalAlign::*;
    match vertical_alignment {
        Top => None,
        Center => Some(0.5), // move the line by the half width
        Bottom => Some(1.0), // move the line by the full width
    }
}

/// Same as `TextLayoutOptions`, but with the widths / heights of the `PixelValue`s
/// resolved to regular f32s (because `letter_spacing`, `word_spacing`, etc. may be %-based value)
#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
#[repr(C)]
pub struct ResolvedTextLayoutOptions {
    /// Font size (in pixels) that this text has been laid out with
    pub font_size_px: f32,
    /// Multiplier for the line height, default to 1.0
    pub line_height: Option<f32>,
    /// Additional spacing between glyphs (in pixels)
    pub letter_spacing: Option<f32>,
    /// Additional spacing between words (in pixels)
    pub word_spacing: Option<f32>,
    /// How many spaces should a tab character emulate
    /// (multiplying value, i.e. `4.0` = one tab = 4 spaces)?
    pub tab_width: Option<f32>,
    /// Maximum width of the text (in pixels) - if the text is set to `overflow:visible`, set this to None.
    pub max_horizontal_width: Option<f32>,
    /// How many pixels of leading does the first line have? Note that this added onto to the holes,
    /// so for effects like `:first-letter`, use a hole instead of a leading.
    pub leading: Option<f32>,
    /// This is more important for inline text layout where items can punch "holes"
    /// into the text flow, for example an image that floats to the right.
    ///
    /// TODO: Currently unused!
    pub holes: Vec<LogicalRect>,
}
