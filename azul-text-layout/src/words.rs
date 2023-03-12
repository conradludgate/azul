use core::fmt;

use crate::{
    logical::{LogicalPosition, LogicalRect, LogicalSize},
    ui_solver::{InlineTextLine, ResolvedTextLayoutOptions},
};

/// Word that is scaled (to a font / font instance), but not yet positioned
#[derive(PartialEq, PartialOrd, Clone)]
#[repr(C)]
pub struct ShapedWord {
    /// Glyph codepoint, glyph ID + kerning data
    pub glyph_infos: Vec<GlyphInfo>,
    /// The sum of the width of all the characters in this word
    pub word_width: usize,
}

impl fmt::Debug for ShapedWord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ShapedWord {{ glyph_infos: {} glyphs, word_width: {} }}",
            self.glyph_infos.len(),
            self.word_width
        )
    }
}

impl ShapedWord {
    pub fn get_word_width(&self, units_per_em: u16, target_font_size: f32) -> f32 {
        self.word_width as f32 / units_per_em as f32 * target_font_size
    }
    /// Returns the number of glyphs THAT ARE NOT DIACRITIC MARKS
    pub fn number_of_glyphs(&self) -> usize {
        self.glyph_infos
            .iter()
            .filter(|i| i.placement == Placement::None)
            .count()
    }
}

/// Stores the positions of the vertically laid out texts
#[derive(Debug, Clone, PartialEq)]
pub struct WordPositions {
    /// Options like word spacing, character spacing, etc. that were
    /// used to layout these glyphs
    pub text_layout_options: ResolvedTextLayoutOptions,
    /// Stores the positions of words.
    pub word_positions: Vec<WordPosition>,
    /// Index of the word at which the line breaks + length of line
    /// (useful for text selection + horizontal centering)
    pub line_breaks: Vec<InlineTextLine>,
    /// Horizontal width of the last line (in pixels), necessary for inline layout later on,
    /// so that the next text run can contine where the last text run left off.
    ///
    /// Usually, the "trailing" of the current text block is the "leading" of the
    /// next text block, to make it seem like two text runs push into each other.
    pub trailing: f32,
    /// How many words are in the text?
    pub number_of_shaped_words: usize,
    /// How many lines (NOTE: virtual lines, meaning line breaks in the layouted text) are there?
    pub number_of_lines: usize,
    /// Horizontal and vertical boundaries of the layouted words.
    ///
    /// Note that the vertical extent can be larger than the last words' position,
    /// because of trailing negative glyph advances.
    pub content_size: LogicalSize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WordPosition {
    pub shaped_word_index: Option<usize>,
    pub position: LogicalPosition,
    pub size: LogicalSize,
}

/// Returns the layouted glyph instances
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutedGlyphs {
    pub glyphs: Vec<GlyphInstance>,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd)]
pub struct GlyphInstance {
    pub index: u32,
    pub point: LogicalPosition,
    pub size: LogicalSize,
}

/// Text broken up into `Tab`, `Word()`, `Return` characters
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Words {
    /// Words (and spaces), broken up into semantic items
    pub items: Vec<Word>,
    /// String that makes up this paragraph of words
    pub internal_str: String,
    // /// `internal_chars` is used in order to enable copy-paste (since taking a sub-string isn't possible using UTF-8)
    // pub internal_chars: Vec<char>U32Vec,
}

impl Words {
    pub fn get_substr(&self, word: &Word) -> &str {
        &self.internal_str.as_str()[word.start..word.end]
    }

    pub fn get_str(&self) -> &str {
        self.internal_str.as_str()
    }

    // pub fn get_char(&self, idx: usize) -> Option<char> {
    //     self.internal_str.as_ref().get(idx).and_then(|c| core::char::from_u32(*c))
    // }
}

/// Section of a certain type
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Word {
    pub start: usize,
    pub end: usize,
    pub word_type: WordType,
}

/// Either a white-space delimited word, tab or return character
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum WordType {
    /// Encountered a word (delimited by spaces)
    Word,
    // `\t` or `x09`
    Tab,
    /// `\r`, `\n` or `\r\n`, escaped: `\x0D`, `\x0A` or `\x0D\x0A`
    Return,
    /// Space character
    Space,
}

/// A paragraph of words that are shaped and scaled (* but not yet layouted / positioned*!)
/// according to their final size in pixels.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct ShapedWords {
    /// Words scaled to their appropriate font size, but not yet positioned on the screen
    pub items: Vec<ShapedWord>,
    /// Longest word in the `self.scaled_words`, necessary for
    /// calculating overflow rectangles.
    pub longest_word_width: usize,
    /// Horizontal advance of the space glyph
    pub space_advance: usize,
    /// Units per EM square
    pub font_metrics_units_per_em: u16,
    /// Descender of the font
    pub font_metrics_ascender: i16,
    pub font_metrics_descender: i16,
    pub font_metrics_line_gap: i16,
}

impl ShapedWords {
    pub fn get_longest_word_width_px(&self, target_font_size: f32) -> f32 {
        self.longest_word_width as f32 / self.font_metrics_units_per_em as f32 * target_font_size
    }
    pub fn get_space_advance_px(&self, target_font_size: f32) -> f32 {
        self.space_advance as f32 / self.font_metrics_units_per_em as f32 * target_font_size
    }
    /// Get the distance from the top of the text to the baseline of the text (= ascender)
    pub fn get_baseline_px(&self, target_font_size: f32) -> f32 {
        target_font_size + self.get_descender(target_font_size)
    }
    /// NOTE: descender is NEGATIVE
    pub fn get_descender(&self, target_font_size: f32) -> f32 {
        self.font_metrics_descender as f32 / self.font_metrics_units_per_em as f32
            * target_font_size
    }

    /// `height = sTypoAscender - sTypoDescender + sTypoLineGap`
    pub fn get_line_height(&self, target_font_size: f32) -> f32 {
        self.font_metrics_ascender as f32 / self.font_metrics_units_per_em as f32
            - self.font_metrics_descender as f32 / self.font_metrics_units_per_em as f32
            + self.font_metrics_line_gap as f32 / self.font_metrics_units_per_em as f32
                * target_font_size
    }

    pub fn get_ascender(&self, target_font_size: f32) -> f32 {
        self.font_metrics_ascender as f32 / self.font_metrics_units_per_em as f32 * target_font_size
    }
}

pub fn get_inline_text(
    words: &Words,
    shaped_words: &ShapedWords,
    word_positions: &WordPositions,
    inline_text_layout: &crate::ui_solver::InlineTextLayout,
) -> InlineText {
    // check the range so that in the worst case there isn't a random crash here
    fn get_range_checked_inclusive_end(
        input: &[Word],
        word_start: usize,
        word_end: usize,
    ) -> Option<&[Word]> {
        if word_start < input.len() && word_end < input.len() && word_start <= word_end {
            Some(&input[word_start..=word_end])
        } else {
            None
        }
    }

    let font_size_px = word_positions.text_layout_options.font_size_px;
    let descender_px = &shaped_words.get_descender(font_size_px); // descender is NEGATIVE
    let letter_spacing_px = word_positions
        .text_layout_options
        .letter_spacing
        .as_ref()
        .copied()
        .unwrap_or(0.0);
    let units_per_em = shaped_words.font_metrics_units_per_em;

    let inline_lines = inline_text_layout
        .lines
        .iter()
        .filter_map(|line| {
            let word_items = words.items.as_ref();
            let word_start = line.word_start.min(line.word_end);
            let word_end = line.word_end.max(line.word_start);

            let words = get_range_checked_inclusive_end(word_items, word_start, word_end)?
                .iter()
                .enumerate()
                .filter_map(|(word_idx, word)| {
                    let word_idx = word_start + word_idx;
                    match word.word_type {
                        WordType::Word => {
                            let word_position = word_positions.word_positions.get(word_idx)?;
                            let shaped_word_index = word_position.shaped_word_index?;
                            let shaped_word = shaped_words.items.get(shaped_word_index)?;

                            // most words are less than 16 chars, avg length of an english word is 4.7 chars
                            let mut all_glyphs_in_this_word = Vec::<InlineGlyph>::with_capacity(16);
                            let mut x_pos_in_word_px = 0.0;

                            // all words only store the unscaled horizontal advance + horizontal kerning
                            for glyph_info in shaped_word.glyph_infos.iter() {
                                // local x and y displacement of the glyph - does NOT advance the horizontal cursor!
                                let mut displacement = LogicalPosition::zero();

                                // if the character is a mark, the mark displacement has to be added ON TOP OF the existing displacement
                                // the origin should be relative to the word, not the final text
                                let (letter_spacing_for_glyph, origin) = match glyph_info.placement
                                {
                                    Placement::None => (
                                        letter_spacing_px,
                                        LogicalPosition::new(
                                            x_pos_in_word_px + displacement.x,
                                            displacement.y,
                                        ),
                                    ),
                                    Placement::Distance(PlacementDistance { x, y }) => {
                                        let font_metrics_divisor =
                                            units_per_em as f32 / font_size_px;
                                        displacement = LogicalPosition {
                                            x: x as f32 / font_metrics_divisor,
                                            y: y as f32 / font_metrics_divisor,
                                        };
                                        (
                                            letter_spacing_px,
                                            LogicalPosition::new(
                                                x_pos_in_word_px + displacement.x,
                                                displacement.y,
                                            ),
                                        )
                                    }
                                    Placement::MarkAnchor(MarkAnchorPlacement {
                                        base_glyph_index,
                                        ..
                                    }) => {
                                        let anchor = &all_glyphs_in_this_word[base_glyph_index];
                                        (0.0, anchor.bounds.origin + displacement)
                                        // TODO: wrong
                                    }
                                    Placement::MarkOverprint(index) => {
                                        let anchor = &all_glyphs_in_this_word[index];
                                        (0.0, anchor.bounds.origin + displacement)
                                    }
                                    Placement::CursiveAnchor(CursiveAnchorPlacement {
                                        exit_glyph_index,
                                        ..
                                    }) => {
                                        let anchor = &all_glyphs_in_this_word[exit_glyph_index];
                                        (0.0, anchor.bounds.origin + displacement)
                                        // TODO: wrong
                                    }
                                };

                                let glyph_scale_x = glyph_info
                                    .size
                                    .get_x_size_scaled(units_per_em, font_size_px);
                                let glyph_scale_y = glyph_info
                                    .size
                                    .get_y_size_scaled(units_per_em, font_size_px);

                                let glyph_advance_x = glyph_info
                                    .size
                                    .get_x_advance_scaled(units_per_em, font_size_px);
                                let kerning_x = glyph_info
                                    .size
                                    .get_kerning_scaled(units_per_em, font_size_px);

                                let inline_char = InlineGlyph {
                                    bounds: LogicalRect::new(
                                        origin,
                                        LogicalSize::new(glyph_scale_x, glyph_scale_y),
                                    ),
                                    unicode_codepoint: glyph_info.glyph.unicode_codepoint,
                                    glyph_index: glyph_info.glyph.glyph_index as u32,
                                };

                                x_pos_in_word_px +=
                                    glyph_advance_x + kerning_x + letter_spacing_for_glyph;

                                all_glyphs_in_this_word.push(inline_char);
                            }

                            let inline_word = InlineWord::Word(InlineTextContents {
                                glyphs: all_glyphs_in_this_word,
                                bounds: LogicalRect::new(
                                    word_position.position,
                                    word_position.size,
                                ),
                            });

                            Some(inline_word)
                        }
                        WordType::Tab => Some(InlineWord::Tab),
                        WordType::Return => Some(InlineWord::Return),
                        WordType::Space => Some(InlineWord::Space),
                    }
                })
                .collect::<Vec<InlineWord>>();

            Some(InlineLine {
                words: words,
                bounds: line.bounds,
            })
        })
        .collect::<Vec<InlineLine>>();

    InlineText {
        lines: inline_lines, // relative to 0, 0
        content_size: word_positions.content_size,
        font_size_px,
        last_word_index: word_positions.number_of_shaped_words,
        baseline_descender_px: *descender_px,
    }
}

/// inline text so that hit-testing is easier
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[repr(C)]
pub struct InlineText {
    /// List of lines, relative to (0.0, 0.0) representing the top left corner of the line
    pub lines: Vec<InlineLine>,
    /// Size of the text content, may be larger than the
    /// position of lines due to descending glyphs
    pub content_size: LogicalSize,
    /// Size of the font used to layout this line
    pub font_size_px: f32,
    /// Index of the last word
    pub last_word_index: usize,
    /// NOTE: descender is NEGATIVE (pixels from baseline to font size)
    pub baseline_descender_px: f32,
}

impl InlineText {
    /// Returns the final, positioned glyphs from an inline text
    ///
    /// NOTE: It seems that at least in webrender, the glyphs have to be
    /// positioned in relation to the screen (instead of relative to the parent container)
    ///
    /// The text_origin gets added to each glyph
    ///
    /// NOTE: The lines in the text are relative to the TOP left corner (of the text, i.e.
    /// relative to the text_origin), but the word position is relative to the BOTTOM left
    /// corner (of the line bounds)
    pub fn get_layouted_glyphs(&self) -> LayoutedGlyphs {
        // descender_px is NEGATIVE
        let baseline_descender_px = LogicalPosition::new(0.0, self.baseline_descender_px);

        LayoutedGlyphs {
            glyphs: self
                .lines
                .iter()
                .flat_map(move |line| {
                    // bottom left corner of line rect
                    let line_origin = line.bounds.origin;

                    line.words.iter().flat_map(move |word| {
                        let (glyphs, mut word_origin) = match word {
                            InlineWord::Tab | InlineWord::Return | InlineWord::Space => {
                                ([].as_slice(), LogicalPosition::zero())
                            }
                            InlineWord::Word(text_contents) => {
                                (text_contents.glyphs.as_slice(), text_contents.bounds.origin)
                            }
                        };

                        word_origin.y = 0.0;

                        glyphs.iter().map(move |glyph| GlyphInstance {
                            index: glyph.glyph_index,
                            point: {
                                line_origin
                                    + baseline_descender_px
                                    + word_origin
                                    + glyph.bounds.origin
                            },
                            size: glyph.bounds.size,
                        })
                    })
                })
                .collect::<Vec<GlyphInstance>>(),
        }
    }

    /// Hit tests all glyphs, returns the hit glyphs - note that the result may
    /// be empty (no glyphs hit), or it may contain more than one result
    /// (overlapping glyphs - more than one glyph hit)
    ///
    /// Usually the result will contain a single `InlineTextHit`
    pub fn hit_test(&self, position: LogicalPosition) -> Vec<InlineTextHit> {
        let bounds = LogicalRect::new(LogicalPosition::zero(), self.content_size);

        let hit_relative_to_inline_text = match bounds.hit_test(&position) {
            Some(s) => s,
            None => return Vec::new(),
        };

        let mut global_char_hit = 0;
        let mut global_word_hit = 0;
        let mut global_glyph_hit = 0;
        let mut global_text_content_hit = 0;

        // NOTE: this function cannot exit early, since it has to
        // iterate through all lines

        let descender_px = self.baseline_descender_px;

        self.lines
        .iter() // TODO: par_iter
        .enumerate()
        .flat_map(|(line_index, line)| {

            let char_at_line_start = global_char_hit;
            let word_at_line_start = global_word_hit;
            let glyph_at_line_start = global_glyph_hit;
            let text_content_at_line_start = global_text_content_hit;

            let mut line_bounds = line.bounds;
            line_bounds.origin.y -= line.bounds.size.height;

            line_bounds.hit_test(&hit_relative_to_inline_text)
            .map(|hit_relative_to_line| {

                line.words
                .iter() // TODO: par_iter
                .flat_map(|word| {

                    let char_at_text_content_start = global_char_hit;
                    let glyph_at_text_content_start = global_glyph_hit;

                    let word_result = word
                    .get_text_content()
                    .and_then(|text_content| {

                        let mut text_content_bounds = text_content.bounds;
                        text_content_bounds.origin.y = 0.0;

                        text_content_bounds
                        .hit_test(&hit_relative_to_line)
                        .map(|hit_relative_to_text_content| {

                            text_content.glyphs
                            .iter() // TODO: par_iter
                            .flat_map(|glyph| {

                                let mut glyph_bounds = glyph.bounds;
                                glyph_bounds.origin.y = text_content.bounds.size.height + descender_px - glyph.bounds.size.height;

                                let result = glyph_bounds
                                .hit_test(&hit_relative_to_text_content)
                                .map(|hit_relative_to_glyph| {
                                    InlineTextHit {
                                        unicode_codepoint: glyph.unicode_codepoint,

                                        hit_relative_to_inline_text,
                                        hit_relative_to_line,
                                        hit_relative_to_text_content,
                                        hit_relative_to_glyph,

                                        line_index_relative_to_text: line_index,
                                        word_index_relative_to_text: global_word_hit,
                                        text_content_index_relative_to_text: global_text_content_hit,
                                        glyph_index_relative_to_text: global_glyph_hit,
                                        char_index_relative_to_text: global_char_hit,

                                        word_index_relative_to_line: global_word_hit - word_at_line_start,
                                        text_content_index_relative_to_line: global_text_content_hit - text_content_at_line_start,
                                        glyph_index_relative_to_line: global_glyph_hit - glyph_at_line_start,
                                        char_index_relative_to_line: global_char_hit - char_at_line_start,

                                        glyph_index_relative_to_word: global_glyph_hit - glyph_at_text_content_start,
                                        char_index_relative_to_word: global_char_hit - char_at_text_content_start,
                                    }
                                });

                                if glyph.has_codepoint() {
                                    global_char_hit += 1;
                                }

                                global_glyph_hit += 1;

                                result
                            })
                            .collect::<Vec<_>>()
                        })
                    }).unwrap_or_default();

                    if word.has_text_content() {
                        global_text_content_hit += 1;
                    }

                    global_word_hit += 1;

                    word_result.into_iter()
                })
                .collect::<Vec<_>>()
            })
            .unwrap_or_default()
            .into_iter()

        })
        .collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[repr(C)]
pub struct InlineLine {
    pub words: Vec<InlineWord>,
    pub bounds: LogicalRect,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[repr(C, u8)]
pub enum InlineWord {
    Tab,
    Return,
    Space,
    Word(InlineTextContents),
}

impl InlineWord {
    pub fn has_text_content(&self) -> bool {
        self.get_text_content().is_some()
    }
    pub fn get_text_content(&self) -> Option<&InlineTextContents> {
        match self {
            InlineWord::Tab | InlineWord::Return | InlineWord::Space => None,
            InlineWord::Word(tc) => Some(tc),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct InlineGlyph {
    pub bounds: LogicalRect,
    pub unicode_codepoint: Option<char>,
    pub glyph_index: u32,
}

impl InlineGlyph {
    pub fn has_codepoint(&self) -> bool {
        self.unicode_codepoint.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct InlineTextContents {
    pub glyphs: Vec<InlineGlyph>,
    pub bounds: LogicalRect,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct InlineTextHit {
    // if the unicode_codepoint is None, it's usually a mark glyph that was hit
    pub unicode_codepoint: Option<char>,

    // position of the cursor relative to X
    pub hit_relative_to_inline_text: LogicalPosition,
    pub hit_relative_to_line: LogicalPosition,
    pub hit_relative_to_text_content: LogicalPosition,
    pub hit_relative_to_glyph: LogicalPosition,

    // relative to text
    pub line_index_relative_to_text: usize,
    pub word_index_relative_to_text: usize,
    pub text_content_index_relative_to_text: usize,
    pub glyph_index_relative_to_text: usize,
    pub char_index_relative_to_text: usize,

    // relative to line
    pub word_index_relative_to_line: usize,
    pub text_content_index_relative_to_line: usize,
    pub glyph_index_relative_to_line: usize,
    pub char_index_relative_to_line: usize,

    // relative to text content (word)
    pub glyph_index_relative_to_word: usize,
    pub char_index_relative_to_word: usize,
}

#[derive(Debug, Copy, PartialEq, PartialOrd, Clone, Hash)]
#[repr(C)]
pub struct PlacementDistance {
    pub x: i32,
    pub y: i32,
}

/// When not Attachment::None indicates that this glyph
/// is an attachment with placement indicated by the variant.
#[derive(Debug, Copy, PartialEq, PartialOrd, Clone, Hash)]
#[repr(C, u8)]
pub enum Placement {
    None,
    Distance(PlacementDistance),
    MarkAnchor(MarkAnchorPlacement),
    /// An overprint mark.
    ///
    /// This mark is shown at the same position as the base glyph.
    ///
    /// Fields: (base glyph index in `Vec<GlyphInfo>`)
    MarkOverprint(usize),
    CursiveAnchor(CursiveAnchorPlacement),
}

/// Cursive anchored placement.
///
/// https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#lookup-type-3-cursive-attachment-positioning-subtable
#[derive(Debug, Copy, PartialEq, PartialOrd, Clone, Hash)]
#[repr(C)]
pub struct CursiveAnchorPlacement {
    /// exit glyph index in the `Vec<GlyphInfo>`
    pub exit_glyph_index: usize,
    /// RIGHT_TO_LEFT flag from lookup table
    pub right_to_left: bool,
    /// exit glyph anchor
    pub exit_glyph_anchor: Anchor,
    /// entry glyph anchor
    pub entry_glyph_anchor: Anchor,
}

/// An anchored mark.
///
/// This is a mark where its anchor is aligned with the base glyph anchor.
#[derive(Debug, Copy, PartialEq, PartialOrd, Clone, Hash)]
#[repr(C)]
pub struct MarkAnchorPlacement {
    /// base glyph index in `Vec<GlyphInfo>`
    pub base_glyph_index: usize,
    /// base glyph anchor
    pub base_glyph_anchor: Anchor,
    /// mark anchor
    pub mark_anchor: Anchor,
}

#[derive(Debug, Copy, PartialEq, PartialOrd, Clone, Hash)]
#[repr(C)]
pub struct Anchor {
    pub x: i16,
    pub y: i16,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Hash)]
pub struct GlyphInfo {
    pub glyph: RawGlyph,
    pub size: Advance,
    pub kerning: i16,
    pub placement: Placement,
}

#[derive(Debug, Copy, PartialEq, PartialOrd, Clone, Hash)]
#[repr(C)]
pub struct RawGlyph {
    pub unicode_codepoint: Option<char>,
    pub glyph_index: u16,
    pub liga_component_pos: u16,
    pub glyph_origin: GlyphOrigin,
    pub small_caps: bool,
    pub multi_subst_dup: bool,
    pub is_vert_alt: bool,
    pub fake_bold: bool,
    pub fake_italic: bool,
    pub variation: Option<VariationSelector>,
}

impl RawGlyph {
    pub fn has_codepoint(&self) -> bool {
        self.unicode_codepoint.is_some()
    }

    pub fn get_codepoint(&self) -> Option<char> {
        self.unicode_codepoint
    }
}

#[derive(Debug, Default, Copy, PartialEq, PartialOrd, Clone, Hash)]
pub struct Advance {
    pub advance_x: u16,
    pub size_x: i32,
    pub size_y: i32,
    pub kerning: i16,
}

impl Advance {
    #[inline]
    pub const fn get_x_advance_total_unscaled(&self) -> i32 {
        self.advance_x as i32 + self.kerning as i32
    }
    #[inline]
    pub const fn get_x_advance_unscaled(&self) -> u16 {
        self.advance_x
    }
    #[inline]
    pub const fn get_x_size_unscaled(&self) -> i32 {
        self.size_x
    }
    #[inline]
    pub const fn get_y_size_unscaled(&self) -> i32 {
        self.size_y
    }
    #[inline]
    pub const fn get_kerning_unscaled(&self) -> i16 {
        self.kerning
    }

    #[inline]
    pub fn get_x_advance_total_scaled(&self, units_per_em: u16, target_font_size: f32) -> f32 {
        self.get_x_advance_total_unscaled() as f32 / units_per_em as f32 * target_font_size
    }
    #[inline]
    pub fn get_x_advance_scaled(&self, units_per_em: u16, target_font_size: f32) -> f32 {
        self.get_x_advance_unscaled() as f32 / units_per_em as f32 * target_font_size
    }
    #[inline]
    pub fn get_x_size_scaled(&self, units_per_em: u16, target_font_size: f32) -> f32 {
        self.get_x_size_unscaled() as f32 / units_per_em as f32 * target_font_size
    }
    #[inline]
    pub fn get_y_size_scaled(&self, units_per_em: u16, target_font_size: f32) -> f32 {
        self.get_y_size_unscaled() as f32 / units_per_em as f32 * target_font_size
    }
    #[inline]
    pub fn get_kerning_scaled(&self, units_per_em: u16, target_font_size: f32) -> f32 {
        self.get_kerning_unscaled() as f32 / units_per_em as f32 * target_font_size
    }
}

/// A Unicode variation selector.
///
/// VS04-VS14 are omitted as they aren't currently used.
#[derive(Debug, Copy, PartialEq, PartialOrd, Clone, Hash)]
pub enum VariationSelector {
    /// VARIATION SELECTOR-1
    VS01 = 1,
    /// VARIATION SELECTOR-2
    VS02 = 2,
    /// VARIATION SELECTOR-3
    VS03 = 3,
    /// Text presentation
    VS15 = 15,
    /// Emoji presentation
    VS16 = 16,
}

#[derive(Debug, Copy, PartialEq, PartialOrd, Clone, Hash)]
pub enum GlyphOrigin {
    Char(char),
    Direct,
}
