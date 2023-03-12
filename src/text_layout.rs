//! Contains functions for breaking a string into words, calculate
//! the positions of words / lines and do glyph positioning

use crate::logical::{LogicalPosition, LogicalRect, LogicalSize};
use crate::text_shaping::ParsedFont;
use crate::{
    ui_solver::{
        InlineTextLayout, InlineTextLine, ResolvedTextLayoutOptions, DEFAULT_LINE_HEIGHT,
        DEFAULT_WORD_SPACING,
    },
    words::{ShapedWord, ShapedWords, Token, Word, WordPosition, WordPositions, Words},
};

/// Creates a font from a font file (TTF, OTF, WOFF, etc.)
///
/// NOTE: EXPENSIVE function, needs to parse tables, etc.
pub fn parse_font(font_bytes: &[u8], font_index: usize) -> Option<ParsedFont> {
    ParsedFont::from_bytes(font_bytes, font_index)
}

/// Splits the text by whitespace into logical units (word, tab, return, whitespace).
pub fn split_text_into_words(text: &str) -> Words {
    use unicode_normalization::UnicodeNormalization;

    // Necessary because we need to handle both \n and \r\n characters
    // If we just look at the characters one-by-one, this wouldn't be possible.
    let normalized_string = text.nfc().collect::<String>();

    let mut words = Vec::new();

    // Instead of storing the actual word, the word is only stored as an index instead,
    // which reduces allocations and is important for later on introducing RTL text
    // (where the position of the character data does not correspond to the actual glyph order).
    let mut current_word_start = 0;
    let mut last_char_idx = 0;
    let mut last_char = '0';
    let mut last_char_was_whitespace = false;

    for (ch_idx, ch) in normalized_string.char_indices() {
        let current_char_is_whitespace = ch == ' ' || ch == '\r' || ch == '\n';

        let should_push_delimiter = match ch {
            ' ' => Some(Word {
                index: (last_char_idx + 1)..(ch_idx + 1),
                word_type: Token::Space,
            }),
            '\n' => {
                Some(if last_char == '\r' {
                    // "\r\n" return
                    Word {
                        index: (last_char_idx)..(ch_idx + 1),
                        word_type: Token::Return,
                    }
                } else {
                    // "\n" return
                    Word {
                        index: (last_char_idx + 1)..(ch_idx + 1),
                        word_type: Token::Return,
                    }
                })
            }
            _ => None,
        };

        // Character is a whitespace or the character is the last character in the text (end of text)
        let should_push_word = if current_char_is_whitespace && !last_char_was_whitespace {
            Some(Word {
                index: (current_word_start)..(ch_idx),
                word_type: Token::Word,
            })
        } else {
            None
        };

        if current_char_is_whitespace {
            current_word_start = ch_idx + 1;
        }

        let mut push_words = |arr: [Option<Word>; 2]| {
            words.extend(arr.into_iter().flatten());
        };

        push_words([should_push_word, should_push_delimiter]);

        last_char_was_whitespace = current_char_is_whitespace;
        last_char_idx = ch_idx;
        last_char = ch;
    }

    // Push the last word
    if current_word_start != last_char_idx + 1 {
        words.push(Word {
            index: current_word_start..normalized_string.len(),
            word_type: Token::Word,
        });
    }

    // If the last item is a `Return`, remove it
    if let Some(Word {
        word_type: Token::Return,
        ..
    }) = words.last()
    {
        words.pop();
    }

    Words {
        items: words,
        internal_str: normalized_string,
        // internal_chars: normalized_chars.iter().map(|c| *c as u32).collect(),
    }
}

/// Takes a text broken into semantic items and shape all the words
/// (does NOT scale the words, only shapes them)
pub fn shape_words(words: &Words, font: &ParsedFont) -> ShapedWords {
    // Get the dimensions of the space glyph
    let space_advance = font
        .get_space_width()
        .unwrap_or(font.font_metrics.head.units_per_em as usize);

    let mut longest_word_width = 0_usize;

    // NOTE: This takes the longest part of the entire layout process -- NEED TO PARALLELIZE
    let shaped_words = words
        .items
        .iter()
        .filter(|w| w.word_type == Token::Word)
        .map(|word| {
            let chars = words.internal_str.as_str()[word.index.clone()]
                .chars()
                .collect::<Vec<_>>();
            let shaped_word = font.shape(&chars);
            let word_width = shaped_word.get_word_visual_width_unscaled();

            longest_word_width = longest_word_width.max(word_width);

            ShapedWord {
                glyph_infos: shaped_word.infos,
                word_width,
            }
        })
        .collect();

    ShapedWords {
        items: shaped_words,
        longest_word_width,
        space_advance,
        font_metrics_units_per_em: font.font_metrics.head.units_per_em,
        font_metrics_ascender: font.font_metrics.get_ascender_unscaled(),
        font_metrics_descender: font.font_metrics.get_descender_unscaled(),
        font_metrics_line_gap: font.font_metrics.get_line_gap_unscaled(),
    }
}

/// Positions the words on the screen (does not layout any glyph positions!), necessary for estimating
/// the intrinsic width + height of the text content.
pub fn position_words(
    words: &Words,
    shaped_words: &ShapedWords,
    text_layout_options: &ResolvedTextLayoutOptions,
) -> WordPositions {
    use self::LineCaretIntersection::*;
    use core::f32;

    let font_size_px = text_layout_options.font_size_px;
    let space_advance_px = shaped_words.get_space_advance_px(text_layout_options.font_size_px);
    let word_spacing_px = space_advance_px
        * text_layout_options
            .word_spacing
            .as_ref()
            .copied()
            .unwrap_or(DEFAULT_WORD_SPACING);
    let line_height_px = space_advance_px
        * text_layout_options
            .line_height
            .as_ref()
            .copied()
            .unwrap_or(DEFAULT_LINE_HEIGHT);
    let spacing_multiplier = text_layout_options
        .letter_spacing
        .as_ref()
        .copied()
        .unwrap_or(0.0);

    let mut line_breaks = Vec::new();
    let mut word_positions = Vec::new();
    let mut line_caret_x = text_layout_options.leading.as_ref().copied().unwrap_or(0.0);
    let mut line_caret_y = font_size_px + line_height_px;
    let mut shaped_word_idx = 0;
    let mut last_shaped_word_word_idx = 0;
    let mut last_line_start_idx = 0;

    let last_word_idx = words.items.len().saturating_sub(1);

    // The last word is a bit special: Any text must have at least one line break!
    for (word_idx, word) in words.items.iter().enumerate() {
        match word.word_type {
            Token::Word => {
                // shaped words only contains the actual shaped words, not spaces / tabs / return chars
                let shaped_word = match shaped_words.items.get(shaped_word_idx) {
                    Some(s) => s,
                    None => continue,
                };

                let letter_spacing_px =
                    spacing_multiplier * shaped_word.number_of_glyphs().saturating_sub(1) as f32;

                // Calculate where the caret would be for the next word
                let shaped_word_width = shaped_word.get_word_width(
                    shaped_words.font_metrics_units_per_em,
                    text_layout_options.font_size_px,
                ) + letter_spacing_px;

                // Determine if a line break is necessary
                let caret_intersection = LineCaretIntersection::new(
                    line_caret_x,
                    shaped_word_width,
                    line_caret_y,
                    font_size_px + line_height_px,
                    text_layout_options.max_horizontal_width.as_ref().copied(),
                );

                // Correct and advance the line caret position
                match caret_intersection {
                    NoLineBreak { new_x, new_y } => {
                        word_positions.push(WordPosition {
                            shaped_word_index: Some(shaped_word_idx),
                            position: LogicalPosition::new(line_caret_x, line_caret_y),
                            size: LogicalSize::new(
                                shaped_word_width,
                                font_size_px + line_height_px,
                            ),
                        });
                        line_caret_x = new_x;
                        line_caret_y = new_y;
                    }
                    LineBreak { new_x, new_y } => {
                        // push the line break first
                        line_breaks.push(InlineTextLine {
                            words: last_line_start_idx
                                ..=word_idx.saturating_sub(1).max(last_line_start_idx),
                            bounds: LogicalRect::new(
                                LogicalPosition::new(0.0, line_caret_y),
                                LogicalSize::new(line_caret_x, font_size_px + line_height_px),
                            ),
                        });
                        last_line_start_idx = word_idx;

                        word_positions.push(WordPosition {
                            shaped_word_index: Some(shaped_word_idx),
                            position: LogicalPosition::new(new_x, new_y),
                            size: LogicalSize::new(
                                shaped_word_width,
                                font_size_px + line_height_px,
                            ),
                        });
                        line_caret_x = new_x + shaped_word_width; // add word width for the next word
                        line_caret_y = new_y;
                    }
                }

                shaped_word_idx += 1;
                last_shaped_word_word_idx = word_idx;
            }
            Token::Return => {
                if word_idx != last_word_idx {
                    line_breaks.push(InlineTextLine {
                        words: last_line_start_idx
                            ..=word_idx.saturating_sub(1).max(last_line_start_idx),
                        bounds: LogicalRect::new(
                            LogicalPosition::new(0.0, line_caret_y),
                            LogicalSize::new(line_caret_x, font_size_px + line_height_px),
                        ),
                    });
                    // don't include the return char in the next line again
                    last_line_start_idx = word_idx + 1;
                }
                word_positions.push(WordPosition {
                    shaped_word_index: None,
                    position: LogicalPosition::new(line_caret_x, line_caret_y),
                    size: LogicalSize::new(0.0, font_size_px + line_height_px),
                });
                if word_idx != last_word_idx {
                    line_caret_x = 0.0;
                    line_caret_y = line_caret_y + font_size_px + line_height_px;
                }
            }
            Token::Space => {
                let x_advance = word_spacing_px;

                let caret_intersection = LineCaretIntersection::new(
                    line_caret_x,
                    x_advance, // advance by space / tab width
                    line_caret_y,
                    font_size_px + line_height_px,
                    text_layout_options.max_horizontal_width.as_ref().copied(),
                );

                match caret_intersection {
                    NoLineBreak { new_x, new_y } => {
                        word_positions.push(WordPosition {
                            shaped_word_index: None,
                            position: LogicalPosition::new(line_caret_x, line_caret_y),
                            size: LogicalSize::new(x_advance, font_size_px + line_height_px),
                        });
                        line_caret_x = new_x;
                        line_caret_y = new_y;
                    }
                    LineBreak { new_x, new_y } => {
                        // push the line break before increasing
                        if word_idx != last_word_idx {
                            line_breaks.push(InlineTextLine {
                                words: last_line_start_idx
                                    ..=word_idx.saturating_sub(1).max(last_line_start_idx),
                                bounds: LogicalRect::new(
                                    LogicalPosition::new(0.0, line_caret_y),
                                    LogicalSize::new(line_caret_x, font_size_px + line_height_px),
                                ),
                            });
                            last_line_start_idx = word_idx;
                        }
                        word_positions.push(WordPosition {
                            shaped_word_index: None,
                            position: LogicalPosition::new(line_caret_x, line_caret_y),
                            size: LogicalSize::new(x_advance, font_size_px + line_height_px),
                        });
                        if word_idx != last_word_idx {
                            line_caret_x = new_x; // don't add the space width here when pushing onto new line
                            line_caret_y = new_y;
                        }
                    }
                }
            }
        }
    }

    line_breaks.push(InlineTextLine {
        words: last_line_start_idx..=last_shaped_word_word_idx,
        bounds: LogicalRect::new(
            LogicalPosition::new(0.0, line_caret_y),
            LogicalSize::new(line_caret_x, font_size_px + line_height_px),
        ),
    });

    let longest_line_width = line_breaks
        .iter()
        .map(|line| line.bounds.size.width)
        .fold(0.0_f32, f32::max);

    let content_size_y = line_breaks.len() as f32 * (font_size_px + line_height_px);
    let content_size_x = text_layout_options
        .max_horizontal_width
        .as_ref()
        .copied()
        .unwrap_or(longest_line_width);
    let content_size = LogicalSize::new(content_size_x, content_size_y);

    WordPositions {
        text_layout_options: text_layout_options.clone(),
        trailing: line_caret_x,
        number_of_shaped_words: shaped_word_idx,
        number_of_lines: line_breaks.len(),
        content_size,
        word_positions,
        line_breaks,
    }
}

/// Returns the (left-aligned!) bounding boxes of the indidividual text lines
pub fn word_positions_to_inline_text_layout(word_positions: &WordPositions) -> InlineTextLayout {
    InlineTextLayout {
        lines: word_positions.line_breaks.clone(),
        content_size: word_positions.content_size,
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
enum LineCaretIntersection {
    /// In order to not intersect with any holes, the caret needs to
    /// be advanced to the position x, but can stay on the same line.
    NoLineBreak { new_x: f32, new_y: f32 },
    /// Caret needs to advance X number of lines and be positioned
    /// with a leading of x
    LineBreak { new_x: f32, new_y: f32 },
}

impl LineCaretIntersection {
    #[inline]
    fn new(
        current_x: f32,
        word_width: f32,
        current_y: f32,
        line_height: f32,
        max_width: Option<f32>,
    ) -> Self {
        match max_width {
            None => LineCaretIntersection::NoLineBreak {
                new_x: current_x + word_width,
                new_y: current_y,
            },
            Some(max) => {
                // window smaller than minimum word content: don't break line
                if current_x == 0.0 && max < word_width {
                    LineCaretIntersection::NoLineBreak {
                        new_x: current_x + word_width,
                        new_y: current_y,
                    }
                } else if (current_x + word_width) > max {
                    LineCaretIntersection::LineBreak {
                        new_x: 0.0,
                        new_y: current_y + line_height,
                    }
                } else {
                    LineCaretIntersection::NoLineBreak {
                        new_x: current_x + word_width,
                        new_y: current_y,
                    }
                }
            }
        }
    }
}

#[test]
fn test_split_words() {
    fn print_words(w: &Words) {
        println!("-- string: {:?}", w.get_str());
        for item in &w.items {
            println!(
                "{:?} - ({:?}) = {:?}",
                w.get_substr(item),
                item.index,
                item.word_type
            );
        }
    }

    fn assert_words(expected: &Words, got_words: &Words) {
        for (idx, expected_word) in expected.items.iter().enumerate() {
            let got = got_words.items.get(idx);
            if got != Some(expected_word) {
                println!("expected: ");
                print_words(expected);
                println!("got: ");
                print_words(got_words);
                panic!(
                    "Expected word idx {} - expected: {:#?}, got: {:#?}",
                    idx,
                    Some(expected_word),
                    got
                );
            }
        }
    }

    let ascii_str = String::from("abc def  \nghi\r\njkl");
    let words_ascii = split_text_into_words(&ascii_str);
    let words_ascii_expected = Words {
        internal_str: ascii_str,
        items: vec![
            Word {
                index: 0..3,
                word_type: Token::Word,
            }, // "abc" - (0..3) = Word
            Word {
                index: 3..4,
                word_type: Token::Space,
            }, // "\t" - (3..4) = Tab
            Word {
                index: 4..7,
                word_type: Token::Word,
            }, // "def" - (4..7) = Word
            Word {
                index: 7..8,
                word_type: Token::Space,
            }, // " " - (7..8) = Space
            Word {
                index: 8..9,
                word_type: Token::Space,
            }, // " " - (8..9) = Space
            Word {
                index: 9..10,
                word_type: Token::Return,
            }, // "\n" - (9..10) = Return
            Word {
                index: 10..13,
                word_type: Token::Word,
            }, // "ghi" - (10..13) = Word
            Word {
                index: 13..15,
                word_type: Token::Return,
            }, // "\r\n" - (13..15) = Return
            Word {
                index: 15..18,
                word_type: Token::Word,
            }, // "jkl" - (15..18) = Word
        ],
    };

    assert_words(&words_ascii_expected, &words_ascii);

    let unicode_str = String::from("㌊㌋㌌㌍㌎㌏㌐㌑ ㌒㌓㌔㌕㌖㌗");
    let words_unicode = split_text_into_words(&unicode_str);
    let words_unicode_expected = Words {
        internal_str: unicode_str,
        // internal_chars: string_to_vec(unicode_str),
        items: vec![
            Word {
                index: 0..8,
                word_type: Token::Word,
            }, // "㌊㌋㌌㌍㌎㌏㌐㌑"
            Word {
                index: 8..9,
                word_type: Token::Space,
            }, // " "
            Word {
                index: 9..15,
                word_type: Token::Word,
            }, // "㌒㌓㌔㌕㌖㌗"
        ],
    };

    assert_words(&words_unicode_expected, &words_unicode);

    let single_str = String::from("A");
    let words_single_str = split_text_into_words(&single_str);
    let words_single_str_expected = Words {
        internal_str: single_str,
        // internal_chars: string_to_vec(single_str),
        items: vec![
            Word {
                index: 0..1,
                word_type: Token::Word,
            }, // "A"
        ],
    };

    assert_words(&words_single_str_expected, &words_single_str);
}
