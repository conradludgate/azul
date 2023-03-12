use crate::{
    css::FontMetrics,
    words::{Advance, GlyphInfo},
};
use allsorts::{
    binary::read::ReadScope,
    font_data::FontData,
    layout::{GDEFTable, LayoutCache, GPOS, GSUB},
    tables::cmap::owned::CmapSubtable as OwnedCmapSubtable,
    tables::{
        cmap::CmapSubtable,
        glyf::{GlyfRecord, GlyfTable, Glyph},
        loca::LocaTable,
        FontTableProvider, HeadTable, MaxpTable,
    },
    tinyvec::tiny_vec,
    DOTTED_CIRCLE,
};
use std::collections::btree_map::BTreeMap;
use std::rc::Rc;

fn get_font_metrics(font_bytes: &[u8], font_index: usize) -> FontMetrics {
    let scope = ReadScope::new(font_bytes);
    let font_file = scope.read::<FontData<'_>>().unwrap();
    let provider = font_file.table_provider(font_index).unwrap();
    let font = allsorts::font::Font::new(provider).unwrap().unwrap();

    // read the HHEA table to get the metrics for horizontal layout
    let head = font.head_table().unwrap().unwrap();
    let os2 = font.os2_table().unwrap().unwrap();
    let hhea = font.hhea_table;

    FontMetrics { hhea, head, os2 }
}

pub struct ParsedFont {
    pub font_metrics: FontMetrics,
    pub num_glyphs: u16,
    pub hmtx_data: Box<[u8]>,
    pub maxp_table: MaxpTable,
    pub gsub_cache: LayoutCache<GSUB>,
    pub gpos_cache: LayoutCache<GPOS>,
    pub opt_gdef_table: Option<Rc<GDEFTable>>,
    pub glyph_records_decoded: BTreeMap<u16, OwnedGlyph>,
    pub space_width: Option<usize>,
    pub cmap_subtable: OwnedCmapSubtable,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[repr(C, u8)]
pub enum GlyphOutlineOperation {
    MoveTo(OutlineMoveTo),
    LineTo(OutlineLineTo),
    QuadraticCurveTo(OutlineQuadTo),
    CubicCurveTo(OutlineCubicTo),
    ClosePath,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[repr(C)]
pub struct OutlineMoveTo {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[repr(C)]
pub struct OutlineLineTo {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[repr(C)]
pub struct OutlineQuadTo {
    pub ctrl_1_x: f32,
    pub ctrl_1_y: f32,
    pub end_x: f32,
    pub end_y: f32,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[repr(C)]
pub struct OutlineCubicTo {
    pub ctrl_1_x: f32,
    pub ctrl_1_y: f32,
    pub ctrl_2_x: f32,
    pub ctrl_2_y: f32,
    pub end_x: f32,
    pub end_y: f32,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[repr(C)]
pub struct GlyphOutline {
    pub operations: Vec<GlyphOutlineOperation>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
struct GlyphOutlineBuilder {
    operations: Vec<GlyphOutlineOperation>,
}

impl ttf_parser::OutlineBuilder for GlyphOutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.operations
            .push(GlyphOutlineOperation::MoveTo(OutlineMoveTo { x, y }));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.operations
            .push(GlyphOutlineOperation::LineTo(OutlineLineTo { x, y }));
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.operations
            .push(GlyphOutlineOperation::QuadraticCurveTo(OutlineQuadTo {
                ctrl_1_x: x1,
                ctrl_1_y: y1,
                end_x: x,
                end_y: y,
            }));
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.operations
            .push(GlyphOutlineOperation::CubicCurveTo(OutlineCubicTo {
                ctrl_1_x: x1,
                ctrl_1_y: y1,
                ctrl_2_x: x2,
                ctrl_2_y: y2,
                end_x: x,
                end_y: y,
            }));
    }
    fn close(&mut self) {
        self.operations.push(GlyphOutlineOperation::ClosePath);
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct OwnedGlyphBoundingBox {
    pub max_x: i16,
    pub max_y: i16,
    pub min_x: i16,
    pub min_y: i16,
}

#[derive(Debug, Clone)]
pub struct OwnedGlyph {
    pub bounding_box: OwnedGlyphBoundingBox,
    pub horz_advance: u16,
    pub outline: Option<GlyphOutline>,
}

impl OwnedGlyph {
    fn from_glyph_data(glyph: Glyph, horz_advance: u16) -> Self {
        Self {
            bounding_box: OwnedGlyphBoundingBox {
                max_x: glyph.bounding_box.x_max,
                max_y: glyph.bounding_box.y_max,
                min_x: glyph.bounding_box.x_min,
                min_y: glyph.bounding_box.y_min,
            },
            horz_advance,
            outline: None,
        }
    }
}

impl ParsedFont {
    pub fn from_bytes(font_bytes: &[u8], font_index: usize) -> Option<Self> {
        use allsorts::tag;

        let scope = ReadScope::new(font_bytes);
        let font_file = scope.read::<FontData<'_>>().ok()?;
        let provider = font_file.table_provider(font_index).ok()?;

        let head_data = provider.table_data(tag::HEAD).ok()??.into_owned();
        let head_table = ReadScope::new(&head_data).read::<HeadTable>().ok()?;

        let maxp_data = provider.table_data(tag::MAXP).ok()??.into_owned();
        let maxp_table = ReadScope::new(&maxp_data).read::<MaxpTable>().ok()?;

        let loca_data = provider.table_data(tag::LOCA).ok()??.into_owned();
        let loca_table = ReadScope::new(&loca_data)
            .read_dep::<LocaTable<'_>>((
                maxp_table.num_glyphs as usize,
                head_table.index_to_loc_format,
            ))
            .ok()?;

        let glyf_data = provider.table_data(tag::GLYF).ok()??.into_owned();
        let glyf_table = ReadScope::new(&glyf_data)
            .read_dep::<GlyfTable<'_>>(&loca_table)
            .ok()?;

        let hmtx_data = provider
            .table_data(tag::HMTX)
            .ok()??
            .into_owned()
            .into_boxed_slice();

        let font_metrics = get_font_metrics(font_bytes, font_index);

        // not parsing glyph outlines can save lots of memory
        let glyph_records_decoded = glyf_table
            .records
            .into_iter()
            .enumerate()
            .filter_map(|(glyph_index, mut glyph_record)| {
                if glyph_index > (u16::MAX as usize) {
                    return None;
                }
                glyph_record.parse().ok()?;
                let glyph_index = glyph_index as u16;
                let horz_advance = allsorts::glyph_info::advance(
                    &maxp_table,
                    &font_metrics.hhea,
                    &hmtx_data,
                    glyph_index,
                )
                .unwrap_or_default();

                match glyph_record {
                    GlyfRecord::Empty | GlyfRecord::Present { .. } => None,
                    GlyfRecord::Parsed(g) => {
                        Some((glyph_index, OwnedGlyph::from_glyph_data(g, horz_advance)))
                    }
                }
            })
            .collect::<Vec<_>>();

        let glyph_records_decoded = glyph_records_decoded.into_iter().collect();

        let mut font_data_impl = allsorts::font::Font::new(provider).ok()??;

        // required for font layout: gsub_cache, gpos_cache and gdef_table
        let gsub_cache = font_data_impl.gsub_cache().ok()??;
        let gpos_cache = font_data_impl.gpos_cache().ok()??;
        let opt_gdef_table = font_data_impl.gdef_table().ok().and_then(|o| o);
        let num_glyphs = font_data_impl.num_glyphs();

        let cmap_subtable = ReadScope::new(font_data_impl.cmap_subtable_data())
            .read::<CmapSubtable<'_>>()
            .ok()?
            .to_owned()?;

        let mut font = ParsedFont {
            font_metrics,
            num_glyphs,
            hmtx_data,
            maxp_table,
            gsub_cache,
            gpos_cache,
            opt_gdef_table,
            cmap_subtable,
            glyph_records_decoded,
            space_width: None,
        };

        let space_width = font.get_space_width_internal();
        font.space_width = space_width;

        Some(font)
    }

    fn get_space_width_internal(&mut self) -> Option<usize> {
        let glyph_index = self.lookup_glyph_index(' ' as u32)?;
        allsorts::glyph_info::advance(
            &self.maxp_table,
            &self.font_metrics.hhea,
            &self.hmtx_data,
            glyph_index,
        )
        .ok()
        .map(|s| s as usize)
    }

    /// Returns the width of the space " " character
    #[inline]
    pub const fn get_space_width(&self) -> Option<usize> {
        self.space_width
    }

    pub fn get_horizontal_advance(&self, glyph_index: u16) -> u16 {
        self.glyph_records_decoded
            .get(&glyph_index)
            .map(|gi| gi.horz_advance)
            .unwrap_or_default()
    }

    // get the x and y size of a glyph in unscaled units
    pub fn get_glyph_size(&self, glyph_index: u16) -> Option<(i32, i32)> {
        let g = self.glyph_records_decoded.get(&glyph_index)?;
        let glyph_width = g.bounding_box.max_x as i32 - g.bounding_box.min_x as i32; // width
        let glyph_height = g.bounding_box.max_y as i32 - g.bounding_box.min_y as i32; // height
        Some((glyph_width, glyph_height))
    }

    pub fn shape(&self, text: &[char]) -> ShapedTextBufferUnsized {
        shape(self, text).unwrap_or_default()
    }

    pub fn lookup_glyph_index(&self, c: u32) -> Option<u16> {
        match self.cmap_subtable.map_glyph(c) {
            Ok(Some(c)) => Some(c),
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
pub struct ShapedTextBufferUnsized {
    pub infos: Vec<GlyphInfo>,
}

impl ShapedTextBufferUnsized {
    /// Get the word width in unscaled units (respects kerning)
    pub fn get_word_visual_width_unscaled(&self) -> usize {
        self.infos
            .iter()
            .map(|s| s.get_x_advance_total_unscaled() as usize)
            .sum()
    }
}

fn shape(font: &ParsedFont, text: &[char]) -> Option<ShapedTextBufferUnsized> {
    use allsorts::gpos::apply as gpos_apply;
    use allsorts::gsub::apply as gsub_apply;
    use allsorts::gsub::{FeatureMask, Features};

    // Map glyphs
    //
    // We look ahead in the char stream for variation selectors. If one is found it is used for
    // mapping the current glyph. When a variation selector is reached in the stream it is skipped
    // as it was handled as part of the preceding character.
    let mut chars_iter = text.iter().peekable();
    let mut glyphs = Vec::with_capacity(text.len());

    while let Some(&ch) = chars_iter.next() {
        match allsorts::unicode::VariationSelector::try_from(ch) {
            Ok(_) => {} // filter out variation selectors
            Err(()) => {
                let vs = chars_iter
                    .peek()
                    .and_then(|&&next| allsorts::unicode::VariationSelector::try_from(next).ok());

                let glyph_index = font.lookup_glyph_index(ch as u32).unwrap_or(0);
                glyphs.push(make_raw_glyph(ch, glyph_index, vs));
            }
        }
    }

    const SCRIPT: u32 = allsorts::tag::LATN;
    let dotted_circle_index = font.lookup_glyph_index(DOTTED_CIRCLE as u32).unwrap_or(0);

    // Apply glyph substitution if table is present
    gsub_apply(
        dotted_circle_index,
        &font.gsub_cache,
        font.opt_gdef_table.as_ref().map(Rc::as_ref),
        SCRIPT,
        None,
        &Features::Mask(FeatureMask::empty()),
        font.num_glyphs,
        &mut glyphs,
    )
    .ok()?;

    // Apply glyph positioning if table is present

    let kerning = true;
    let mut infos = allsorts::gpos::Info::init_from_glyphs(
        font.opt_gdef_table.as_ref().map(Rc::as_ref),
        glyphs,
    );

    gpos_apply(
        &font.gpos_cache,
        font.opt_gdef_table.as_ref().map(Rc::as_ref),
        kerning,
        &Features::Mask(FeatureMask::all()),
        SCRIPT,
        None,
        &mut infos,
    )
    .ok()?;

    // calculate the horizontal advance for each char
    let infos = infos
        .into_iter()
        .filter_map(|info| {
            let glyph_index = info.glyph.glyph_index;
            let adv_x = font.get_horizontal_advance(glyph_index);
            let (size_x, size_y) = font.get_glyph_size(glyph_index)?;
            let advance = Advance {
                advance_x: adv_x,
                size_x,
                size_y,
            };
            Some(GlyphInfo { info, advance })
        })
        .collect();

    Some(ShapedTextBufferUnsized { infos })
}

fn make_raw_glyph(
    ch: char,
    glyph_index: u16,
    variation: Option<allsorts::unicode::VariationSelector>,
) -> allsorts::gsub::RawGlyph<()> {
    allsorts::gsub::RawGlyph {
        unicodes: tiny_vec![[char; 1] => ch],
        glyph_index,
        liga_component_pos: 0,
        glyph_origin: allsorts::gsub::GlyphOrigin::Char(ch),
        small_caps: false,
        multi_subst_dup: false,
        is_vert_alt: false,
        fake_bold: false,
        fake_italic: false,
        extra_data: (),
        variation,
    }
}
