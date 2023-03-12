//! Provides a public API with datatypes used to describe style properties of DOM nodes.

use crate::css::CssPropertyValue;
use crate::text_shaping::ParsedFont;
use core::cmp::Ordering;
use core::ffi::c_void;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use std::collections::btree_map::BTreeMap;
use std::sync::Arc;

/// Currently hard-coded: Height of one em in pixels
pub const EM_HEIGHT: f32 = 16.0;
pub const PT_TO_PX: f32 = 96.0 / 72.0;

const COMBINED_CSS_PROPERTIES_KEY_MAP: [(CombinedCssPropertyType, &'static str); 12] = [
    (CombinedCssPropertyType::BorderRadius, "border-radius"),
    (CombinedCssPropertyType::Overflow, "overflow"),
    (CombinedCssPropertyType::Padding, "padding"),
    (CombinedCssPropertyType::Margin, "margin"),
    (CombinedCssPropertyType::Border, "border"),
    (CombinedCssPropertyType::BorderLeft, "border-left"),
    (CombinedCssPropertyType::BorderRight, "border-right"),
    (CombinedCssPropertyType::BorderTop, "border-top"),
    (CombinedCssPropertyType::BorderBottom, "border-bottom"),
    (CombinedCssPropertyType::BoxShadow, "box-shadow"),
    (CombinedCssPropertyType::BackgroundColor, "background-color"),
    (CombinedCssPropertyType::BackgroundImage, "background-image"),
];

/// Map between CSS keys and a statically typed enum
const CSS_PROPERTY_KEY_MAP: [(CssPropertyType, &'static str); 74] = [
    (CssPropertyType::Display, "display"),
    (CssPropertyType::Float, "float"),
    (CssPropertyType::BoxSizing, "box-sizing"),
    (CssPropertyType::TextColor, "color"),
    (CssPropertyType::FontSize, "font-size"),
    (CssPropertyType::FontFamily, "font-family"),
    (CssPropertyType::TextAlign, "text-align"),
    (CssPropertyType::LetterSpacing, "letter-spacing"),
    (CssPropertyType::LineHeight, "line-height"),
    (CssPropertyType::WordSpacing, "word-spacing"),
    (CssPropertyType::TabWidth, "tab-width"),
    (CssPropertyType::Cursor, "cursor"),
    (CssPropertyType::Width, "width"),
    (CssPropertyType::Height, "height"),
    (CssPropertyType::MinWidth, "min-width"),
    (CssPropertyType::MinHeight, "min-height"),
    (CssPropertyType::MaxWidth, "max-width"),
    (CssPropertyType::MaxHeight, "max-height"),
    (CssPropertyType::Position, "position"),
    (CssPropertyType::Top, "top"),
    (CssPropertyType::Right, "right"),
    (CssPropertyType::Left, "left"),
    (CssPropertyType::Bottom, "bottom"),
    (CssPropertyType::FlexWrap, "flex-wrap"),
    (CssPropertyType::FlexDirection, "flex-direction"),
    (CssPropertyType::FlexGrow, "flex-grow"),
    (CssPropertyType::FlexShrink, "flex-shrink"),
    (CssPropertyType::JustifyContent, "justify-content"),
    (CssPropertyType::AlignItems, "align-items"),
    (CssPropertyType::AlignContent, "align-content"),
    (CssPropertyType::OverflowX, "overflow-x"),
    (CssPropertyType::OverflowY, "overflow-y"),
    (CssPropertyType::PaddingTop, "padding-top"),
    (CssPropertyType::PaddingLeft, "padding-left"),
    (CssPropertyType::PaddingRight, "padding-right"),
    (CssPropertyType::PaddingBottom, "padding-bottom"),
    (CssPropertyType::MarginTop, "margin-top"),
    (CssPropertyType::MarginLeft, "margin-left"),
    (CssPropertyType::MarginRight, "margin-right"),
    (CssPropertyType::MarginBottom, "margin-bottom"),
    (CssPropertyType::BackgroundContent, "background"),
    (CssPropertyType::BackgroundPosition, "background-position"),
    (CssPropertyType::BackgroundSize, "background-size"),
    (CssPropertyType::BackgroundRepeat, "background-repeat"),
    (
        CssPropertyType::BorderTopLeftRadius,
        "border-top-left-radius",
    ),
    (
        CssPropertyType::BorderTopRightRadius,
        "border-top-right-radius",
    ),
    (
        CssPropertyType::BorderBottomLeftRadius,
        "border-bottom-left-radius",
    ),
    (
        CssPropertyType::BorderBottomRightRadius,
        "border-bottom-right-radius",
    ),
    (CssPropertyType::BorderTopColor, "border-top-color"),
    (CssPropertyType::BorderRightColor, "border-right-color"),
    (CssPropertyType::BorderLeftColor, "border-left-color"),
    (CssPropertyType::BorderBottomColor, "border-bottom-color"),
    (CssPropertyType::BorderTopStyle, "border-top-style"),
    (CssPropertyType::BorderRightStyle, "border-right-style"),
    (CssPropertyType::BorderLeftStyle, "border-left-style"),
    (CssPropertyType::BorderBottomStyle, "border-bottom-style"),
    (CssPropertyType::BorderTopWidth, "border-top-width"),
    (CssPropertyType::BorderRightWidth, "border-right-width"),
    (CssPropertyType::BorderLeftWidth, "border-left-width"),
    (CssPropertyType::BorderBottomWidth, "border-bottom-width"),
    (CssPropertyType::BoxShadowTop, "-azul-box-shadow-top"),
    (CssPropertyType::BoxShadowRight, "-azul-box-shadow-right"),
    (CssPropertyType::BoxShadowLeft, "-azul-box-shadow-left"),
    (CssPropertyType::BoxShadowBottom, "-azul-box-shadow-bottom"),
    (CssPropertyType::ScrollbarStyle, "-azul-scrollbar-style"),
    (CssPropertyType::Opacity, "opacity"),
    (CssPropertyType::Transform, "transform"),
    (CssPropertyType::PerspectiveOrigin, "perspective-origin"),
    (CssPropertyType::TransformOrigin, "transform-origin"),
    (CssPropertyType::BackfaceVisibility, "backface-visibility"),
    (CssPropertyType::MixBlendMode, "mix-blend-mode"),
    (CssPropertyType::Filter, "filter"),
    (CssPropertyType::BackdropFilter, "backdrop-filter"),
    (CssPropertyType::TextShadow, "text-shadow"),
];

// The following types are present in webrender, however, azul-css should not
// depend on webrender, just to have the same types, azul-css should be a standalone crate.

/// Only used for calculations: Rectangle (x, y, width, height) in layout space.
#[derive(Copy, Clone, PartialEq, PartialOrd)]
#[repr(C)]
pub struct LayoutRect {
    pub origin: LayoutPoint,
    pub size: LayoutSize,
}

impl fmt::Debug for LayoutRect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for LayoutRect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} @ {}", self.size, self.origin)
    }
}

impl LayoutRect {
    #[inline(always)]
    pub const fn new(origin: LayoutPoint, size: LayoutSize) -> Self {
        Self { origin, size }
    }
    #[inline(always)]
    pub const fn zero() -> Self {
        Self::new(LayoutPoint::zero(), LayoutSize::zero())
    }
    #[inline(always)]
    pub const fn max_x(&self) -> isize {
        self.origin.x + self.size.width
    }
    #[inline(always)]
    pub const fn min_x(&self) -> isize {
        self.origin.x
    }
    #[inline(always)]
    pub const fn max_y(&self) -> isize {
        self.origin.y + self.size.height
    }
    #[inline(always)]
    pub const fn min_y(&self) -> isize {
        self.origin.y
    }
    #[inline(always)]
    pub const fn width(&self) -> isize {
        self.max_x() - self.min_x()
    }
    #[inline(always)]
    pub const fn height(&self) -> isize {
        self.max_y() - self.min_y()
    }

    pub const fn contains(&self, other: &LayoutPoint) -> bool {
        self.min_x() <= other.x
            && other.x < self.max_x()
            && self.min_y() <= other.y
            && other.y < self.max_y()
    }

    pub fn contains_f32(&self, other_x: f32, other_y: f32) -> bool {
        self.min_x() as f32 <= other_x
            && other_x < self.max_x() as f32
            && self.min_y() as f32 <= other_y
            && other_y < self.max_y() as f32
    }

    /// Same as `contains()`, but returns the (x, y) offset of the hit point
    ///
    /// On a regular computer this function takes ~3.2ns to run
    #[inline]
    pub const fn hit_test(&self, other: &LayoutPoint) -> Option<LayoutPoint> {
        let dx_left_edge = other.x - self.min_x();
        let dx_right_edge = self.max_x() - other.x;
        let dy_top_edge = other.y - self.min_y();
        let dy_bottom_edge = self.max_y() - other.y;
        if dx_left_edge > 0 && dx_right_edge > 0 && dy_top_edge > 0 && dy_bottom_edge > 0 {
            Some(LayoutPoint::new(dx_left_edge, dy_top_edge))
        } else {
            None
        }
    }

    /// Faster union for a Vec<LayoutRect>
    #[inline]
    pub fn union<I: Iterator<Item = Self>>(mut rects: I) -> Option<Self> {
        let first = rects.next()?;

        let mut max_width = first.size.width;
        let mut max_height = first.size.height;
        let mut min_x = first.origin.x;
        let mut min_y = first.origin.y;

        while let Some(Self {
            origin: LayoutPoint { x, y },
            size: LayoutSize { width, height },
        }) = rects.next()
        {
            let cur_lower_right_x = x + width;
            let cur_lower_right_y = y + height;
            max_width = max_width.max(cur_lower_right_x - min_x);
            max_height = max_height.max(cur_lower_right_y - min_y);
            min_x = min_x.min(x);
            min_y = min_y.min(y);
        }

        Some(Self {
            origin: LayoutPoint { x: min_x, y: min_y },
            size: LayoutSize {
                width: max_width,
                height: max_height,
            },
        })
    }

    // Returns the scroll rect (not the union rect) of the parent / children
    #[inline]
    pub fn get_scroll_rect<I: Iterator<Item = Self>>(&self, children: I) -> Option<Self> {
        let children_union = Self::union(children)?;
        Self::union([*self, children_union].iter().map(|r| *r))
    }

    // Returns if b overlaps a
    #[inline(always)]
    pub const fn contains_rect(&self, b: &LayoutRect) -> bool {
        let a = self;

        let a_x = a.origin.x;
        let a_y = a.origin.y;
        let a_width = a.size.width;
        let a_height = a.size.height;

        let b_x = b.origin.x;
        let b_y = b.origin.y;
        let b_width = b.size.width;
        let b_height = b.size.height;

        b_x >= a_x
            && b_y >= a_y
            && b_x + b_width <= a_x + a_width
            && b_y + b_height <= a_y + a_height
    }
}

/// Only used for calculations: Size (width, height) in layout space.
#[derive(Copy, Default, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
#[repr(C)]
pub struct LayoutSize {
    pub width: isize,
    pub height: isize,
}

impl fmt::Debug for LayoutSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for LayoutSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

impl LayoutSize {
    #[inline(always)]
    pub const fn new(width: isize, height: isize) -> Self {
        Self { width, height }
    }
    #[inline(always)]
    pub const fn zero() -> Self {
        Self::new(0, 0)
    }
    #[inline]
    pub fn round(width: f32, height: f32) -> Self {
        Self {
            width: libm::roundf(width) as isize,
            height: libm::roundf(height) as isize,
        }
    }
}

/// Only used for calculations: Point coordinate (x, y) in layout space.
#[derive(Copy, Default, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
#[repr(C)]
pub struct LayoutPoint {
    pub x: isize,
    pub y: isize,
}

impl fmt::Debug for LayoutPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for LayoutPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl LayoutPoint {
    #[inline(always)]
    pub const fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }
    #[inline(always)]
    pub const fn zero() -> Self {
        Self::new(0, 0)
    }
}

/// Represents a parsed pair of `5px, 10px` values - useful for border radius calculation
#[derive(Default, Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct PixelSize {
    pub width: PixelValue,
    pub height: PixelValue,
}

impl PixelSize {
    pub const fn new(width: PixelValue, height: PixelValue) -> Self {
        Self { width, height }
    }

    pub const fn zero() -> Self {
        Self::new(PixelValue::const_px(0), PixelValue::const_px(0))
    }
}

/// Offsets of the border-width calculations
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[repr(C)]
pub struct LayoutSideOffsets {
    pub top: FloatValue,
    pub right: FloatValue,
    pub bottom: FloatValue,
    pub left: FloatValue,
}

/// u8-based color, range 0 to 255 (similar to webrenders ColorU)
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[repr(C)]
pub struct ColorU {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for ColorU {
    fn default() -> Self {
        ColorU::BLACK
    }
}

impl fmt::Display for ColorU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "rgba({}, {}, {}, {})",
            self.r,
            self.g,
            self.b,
            self.a as f32 / 255.0
        )
    }
}

impl ColorU {
    pub const ALPHA_TRANSPARENT: u8 = 0;
    pub const ALPHA_OPAQUE: u8 = 255;

    pub const RED: ColorU = ColorU {
        r: 255,
        g: 0,
        b: 0,
        a: Self::ALPHA_OPAQUE,
    };
    pub const GREEN: ColorU = ColorU {
        r: 0,
        g: 255,
        b: 0,
        a: Self::ALPHA_OPAQUE,
    };
    pub const BLUE: ColorU = ColorU {
        r: 0,
        g: 0,
        b: 255,
        a: Self::ALPHA_OPAQUE,
    };
    pub const WHITE: ColorU = ColorU {
        r: 255,
        g: 255,
        b: 255,
        a: Self::ALPHA_OPAQUE,
    };
    pub const BLACK: ColorU = ColorU {
        r: 0,
        g: 0,
        b: 0,
        a: Self::ALPHA_OPAQUE,
    };
    pub const TRANSPARENT: ColorU = ColorU {
        r: 0,
        g: 0,
        b: 0,
        a: Self::ALPHA_TRANSPARENT,
    };

    pub const fn new_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        Self {
            r: libm::roundf(self.r as f32 + (other.r as f32 - self.r as f32) * t) as u8,
            g: libm::roundf(self.g as f32 + (other.g as f32 - self.g as f32) * t) as u8,
            b: libm::roundf(self.b as f32 + (other.b as f32 - self.b as f32) * t) as u8,
            a: libm::roundf(self.a as f32 + (other.a as f32 - self.a as f32) * t) as u8,
        }
    }

    pub const fn has_alpha(&self) -> bool {
        self.a != Self::ALPHA_OPAQUE
    }

    pub fn to_hash(&self) -> String {
        format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
    }

    pub fn write_hash(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "#{:02x}{:02x}{:02x}{:02x}",
            self.r, self.g, self.b, self.a
        )
    }
}

/// f32-based color, range 0.0 to 1.0 (similar to webrenders ColorF)
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct ColorF {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for ColorF {
    fn default() -> Self {
        ColorF::BLACK
    }
}

impl fmt::Display for ColorF {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "rgba({}, {}, {}, {})",
            self.r * 255.0,
            self.g * 255.0,
            self.b * 255.0,
            self.a
        )
    }
}

impl ColorF {
    pub const ALPHA_TRANSPARENT: f32 = 0.0;
    pub const ALPHA_OPAQUE: f32 = 1.0;

    pub const WHITE: ColorF = ColorF {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: Self::ALPHA_OPAQUE,
    };
    pub const BLACK: ColorF = ColorF {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: Self::ALPHA_OPAQUE,
    };
    pub const TRANSPARENT: ColorF = ColorF {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: Self::ALPHA_TRANSPARENT,
    };
}

impl From<ColorU> for ColorF {
    fn from(input: ColorU) -> ColorF {
        ColorF {
            r: (input.r as f32) / 255.0,
            g: (input.g as f32) / 255.0,
            b: (input.b as f32) / 255.0,
            a: (input.a as f32) / 255.0,
        }
    }
}

impl From<ColorF> for ColorU {
    fn from(input: ColorF) -> ColorU {
        ColorU {
            r: (input.r.min(1.0) * 255.0) as u8,
            g: (input.g.min(1.0) * 255.0) as u8,
            b: (input.b.min(1.0) * 255.0) as u8,
            a: (input.a.min(1.0) * 255.0) as u8,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub enum BorderDetails {
    Normal(NormalBorder),
    NinePatch(NinePatchBorder),
}

/// Represents a normal `border` property (no image border / nine-patch border)
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct NormalBorder {
    pub left: BorderSide,
    pub right: BorderSide,
    pub top: BorderSide,
    pub bottom: BorderSide,
    pub radius: Option<(
        StyleBorderTopLeftRadius,
        StyleBorderTopRightRadius,
        StyleBorderBottomLeftRadius,
        StyleBorderBottomRightRadius,
    )>,
}

#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[repr(C)]
pub struct BorderSide {
    pub color: ColorU,
    pub style: BorderStyle,
}

/// What direction should a `box-shadow` be clipped in (inset or outset)
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[repr(C)]
pub enum BoxShadowClipMode {
    Outset,
    Inset,
}

impl fmt::Display for BoxShadowClipMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::BoxShadowClipMode::*;
        match self {
            Outset => write!(f, "outset"),
            Inset => write!(f, "inset"),
        }
    }
}

/// Whether a `gradient` should be repeated or clamped to the edges.
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[repr(C)]
pub enum ExtendMode {
    Clamp,
    Repeat,
}

impl Default for ExtendMode {
    fn default() -> Self {
        ExtendMode::Clamp
    }
}

/// Style of a `border`: solid, double, dash, ridge, etc.
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[repr(C)]
pub enum BorderStyle {
    None,
    Solid,
    Double,
    Dotted,
    Dashed,
    Hidden,
    Groove,
    Ridge,
    Inset,
    Outset,
}

impl fmt::Display for BorderStyle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::BorderStyle::*;
        match self {
            None => write!(f, "none"),
            Solid => write!(f, "solid"),
            Double => write!(f, "double"),
            Dotted => write!(f, "dotted"),
            Dashed => write!(f, "dashed"),
            Hidden => write!(f, "hidden"),
            Groove => write!(f, "groove"),
            Ridge => write!(f, "ridge"),
            Inset => write!(f, "inset"),
            Outset => write!(f, "outset"),
        }
    }
}

impl BorderStyle {
    pub fn normalize_border(self) -> Option<BorderStyleNoNone> {
        match self {
            BorderStyle::None => None,
            BorderStyle::Solid => Some(BorderStyleNoNone::Solid),
            BorderStyle::Double => Some(BorderStyleNoNone::Double),
            BorderStyle::Dotted => Some(BorderStyleNoNone::Dotted),
            BorderStyle::Dashed => Some(BorderStyleNoNone::Dashed),
            BorderStyle::Hidden => Some(BorderStyleNoNone::Hidden),
            BorderStyle::Groove => Some(BorderStyleNoNone::Groove),
            BorderStyle::Ridge => Some(BorderStyleNoNone::Ridge),
            BorderStyle::Inset => Some(BorderStyleNoNone::Inset),
            BorderStyle::Outset => Some(BorderStyleNoNone::Outset),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub enum BorderStyleNoNone {
    Solid,
    Double,
    Dotted,
    Dashed,
    Hidden,
    Groove,
    Ridge,
    Inset,
    Outset,
}

impl Default for BorderStyle {
    fn default() -> Self {
        BorderStyle::Solid
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct NinePatchBorder {
    // not implemented or parse-able yet, so no fields!
}

macro_rules! derive_debug_zero {
    ($struct:ident) => {
        impl fmt::Debug for $struct {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{:?}", self.inner)
            }
        }
    };
}

macro_rules! derive_display_zero {
    ($struct:ident) => {
        impl fmt::Display for $struct {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.inner)
            }
        }
    };
}

/// Creates `pt`, `px` and `em` constructors for any struct that has a
/// `PixelValue` as it's self.0 field.
macro_rules! impl_pixel_value {
    ($struct:ident) => {
        derive_debug_zero!($struct);
        derive_display_zero!($struct);

        impl $struct {
            #[inline]
            pub const fn zero() -> Self {
                Self {
                    inner: PixelValue::zero(),
                }
            }

            /// Same as `PixelValue::px()`, but only accepts whole numbers,
            /// since using `f32` in const fn is not yet stabilized.
            #[inline]
            pub const fn const_px(value: isize) -> Self {
                Self {
                    inner: PixelValue::const_px(value),
                }
            }

            /// Same as `PixelValue::em()`, but only accepts whole numbers,
            /// since using `f32` in const fn is not yet stabilized.
            #[inline]
            pub const fn const_em(value: isize) -> Self {
                Self {
                    inner: PixelValue::const_em(value),
                }
            }

            /// Same as `PixelValue::pt()`, but only accepts whole numbers,
            /// since using `f32` in const fn is not yet stabilized.
            #[inline]
            pub const fn const_pt(value: isize) -> Self {
                Self {
                    inner: PixelValue::const_pt(value),
                }
            }

            /// Same as `PixelValue::pt()`, but only accepts whole numbers,
            /// since using `f32` in const fn is not yet stabilized.
            #[inline]
            pub const fn const_percent(value: isize) -> Self {
                Self {
                    inner: PixelValue::const_percent(value),
                }
            }

            #[inline]
            pub const fn const_from_metric(metric: SizeMetric, value: isize) -> Self {
                Self {
                    inner: PixelValue::const_from_metric(metric, value),
                }
            }

            #[inline]
            pub fn px(value: f32) -> Self {
                Self {
                    inner: PixelValue::px(value),
                }
            }

            #[inline]
            pub fn em(value: f32) -> Self {
                Self {
                    inner: PixelValue::em(value),
                }
            }

            #[inline]
            pub fn pt(value: f32) -> Self {
                Self {
                    inner: PixelValue::pt(value),
                }
            }

            #[inline]
            pub fn percent(value: f32) -> Self {
                Self {
                    inner: PixelValue::percent(value),
                }
            }

            #[inline]
            pub fn from_metric(metric: SizeMetric, value: f32) -> Self {
                Self {
                    inner: PixelValue::from_metric(metric, value),
                }
            }

            #[inline]
            pub fn interpolate(&self, other: &Self, t: f32) -> Self {
                $struct {
                    inner: self.inner.interpolate(&other.inner, t),
                }
            }
        }
    };
}

macro_rules! impl_percentage_value {
    ($struct:ident) => {
        impl ::core::fmt::Display for $struct {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                write!(f, "{}%", self.inner.get())
            }
        }

        impl ::core::fmt::Debug for $struct {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                write!(f, "{}%", self.inner.get())
            }
        }

        impl $struct {
            /// Same as `PercentageValue::new()`, but only accepts whole numbers,
            /// since using `f32` in const fn is not yet stabilized.
            #[inline]
            pub const fn const_new(value: isize) -> Self {
                Self {
                    inner: PercentageValue::const_new(value),
                }
            }

            #[inline]
            pub fn new(value: f32) -> Self {
                Self {
                    inner: PercentageValue::new(value),
                }
            }

            #[inline]
            pub fn interpolate(&self, other: &Self, t: f32) -> Self {
                $struct {
                    inner: self.inner.interpolate(&other.inner, t),
                }
            }
        }
    };
}

macro_rules! impl_float_value {
    ($struct:ident) => {
        impl $struct {
            /// Same as `FloatValue::new()`, but only accepts whole numbers,
            /// since using `f32` in const fn is not yet stabilized.
            pub const fn const_new(value: isize) -> Self {
                Self {
                    inner: FloatValue::const_new(value),
                }
            }

            pub fn new(value: f32) -> Self {
                Self {
                    inner: FloatValue::new(value),
                }
            }

            pub fn get(&self) -> f32 {
                self.inner.get()
            }

            #[inline]
            pub fn interpolate(&self, other: &Self, t: f32) -> Self {
                Self {
                    inner: self.inner.interpolate(&other.inner, t),
                }
            }
        }

        impl From<f32> for $struct {
            fn from(val: f32) -> Self {
                Self {
                    inner: FloatValue::from(val),
                }
            }
        }

        impl ::core::fmt::Display for $struct {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                write!(f, "{}", self.inner.get())
            }
        }

        impl ::core::fmt::Debug for $struct {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                write!(f, "{}", self.inner.get())
            }
        }
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CombinedCssPropertyType {
    BorderRadius,
    Overflow,
    Margin,
    Border,
    BorderLeft,
    BorderRight,
    BorderTop,
    BorderBottom,
    Padding,
    BoxShadow,
    BackgroundColor, // BackgroundContent::Colo
    BackgroundImage, // BackgroundContent::Colo
}

impl fmt::Display for CombinedCssPropertyType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let key = COMBINED_CSS_PROPERTIES_KEY_MAP
            .iter()
            .find(|(v, _)| *v == *self)
            .and_then(|(k, _)| Some(k))
            .unwrap();
        write!(f, "{}", key)
    }
}

impl CombinedCssPropertyType {
    /// Parses a CSS key, such as `width` from a string:
    ///
    /// # Example
    ///
    /// ```rust
    /// # use azul_css::{CombinedCssPropertyType, get_css_key_map};
    /// let map = get_css_key_map();
    /// assert_eq!(Some(CombinedCssPropertyType::Border), CombinedCssPropertyType::from_str("border", &map));
    /// ```
    pub fn from_str(input: &str, map: &CssKeyMap) -> Option<Self> {
        let input = input.trim();
        map.shorthands.get(input).map(|x| *x)
    }

    /// Returns the original string that was used to construct this `CssPropertyType`.
    pub fn to_str(&self, map: &CssKeyMap) -> &'static str {
        map.shorthands
            .iter()
            .find(|(_, v)| *v == self)
            .map(|(k, _)| k)
            .unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CssKeyMap {
    // Contains all keys that have no shorthand
    pub non_shorthands: BTreeMap<&'static str, CssPropertyType>,
    // Contains all keys that act as a shorthand for other types
    pub shorthands: BTreeMap<&'static str, CombinedCssPropertyType>,
}

impl CssKeyMap {
    pub fn get() -> Self {
        get_css_key_map()
    }
}

/// Returns a map useful for parsing the keys of CSS stylesheets
pub fn get_css_key_map() -> CssKeyMap {
    CssKeyMap {
        non_shorthands: CSS_PROPERTY_KEY_MAP.iter().map(|(v, k)| (*k, *v)).collect(),
        shorthands: COMBINED_CSS_PROPERTIES_KEY_MAP
            .iter()
            .map(|(v, k)| (*k, *v))
            .collect(),
    }
}

/// Represents a CSS key (for example `"border-radius"` => `BorderRadius`).
/// You can also derive this key from a `CssProperty` by calling `CssProperty::get_type()`.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum CssPropertyType {
    TextColor,
    FontSize,
    FontFamily,
    TextAlign,
    LetterSpacing,
    LineHeight,
    WordSpacing,
    TabWidth,
    Cursor,
    Display,
    Float,
    BoxSizing,
    Width,
    Height,
    MinWidth,
    MinHeight,
    MaxWidth,
    MaxHeight,
    Position,
    Top,
    Right,
    Left,
    Bottom,
    FlexWrap,
    FlexDirection,
    FlexGrow,
    FlexShrink,
    JustifyContent,
    AlignItems,
    AlignContent,
    BackgroundContent,
    BackgroundPosition,
    BackgroundSize,
    BackgroundRepeat,
    OverflowX,
    OverflowY,
    PaddingTop,
    PaddingLeft,
    PaddingRight,
    PaddingBottom,
    MarginTop,
    MarginLeft,
    MarginRight,
    MarginBottom,
    BorderTopLeftRadius,
    BorderTopRightRadius,
    BorderBottomLeftRadius,
    BorderBottomRightRadius,
    BorderTopColor,
    BorderRightColor,
    BorderLeftColor,
    BorderBottomColor,
    BorderTopStyle,
    BorderRightStyle,
    BorderLeftStyle,
    BorderBottomStyle,
    BorderTopWidth,
    BorderRightWidth,
    BorderLeftWidth,
    BorderBottomWidth,
    BoxShadowLeft,
    BoxShadowRight,
    BoxShadowTop,
    BoxShadowBottom,
    ScrollbarStyle,
    Opacity,
    Transform,
    TransformOrigin,
    PerspectiveOrigin,
    BackfaceVisibility,
    MixBlendMode,
    Filter,
    BackdropFilter,
    TextShadow,
}

impl CssPropertyType {
    /// Parses a CSS key, such as `width` from a string:
    ///
    /// # Example
    ///
    /// ```rust
    /// # use azul_css::{CssPropertyType, get_css_key_map};
    /// let map = get_css_key_map();
    /// assert_eq!(Some(CssPropertyType::Width), CssPropertyType::from_str("width", &map));
    /// assert_eq!(Some(CssPropertyType::JustifyContent), CssPropertyType::from_str("justify-content", &map));
    /// assert_eq!(None, CssPropertyType::from_str("asdfasdfasdf", &map));
    /// ```
    pub fn from_str(input: &str, map: &CssKeyMap) -> Option<Self> {
        let input = input.trim();
        map.non_shorthands.get(input).and_then(|x| Some(*x))
    }

    /// Returns the original string that was used to construct this `CssPropertyType`.
    pub fn to_str(&self) -> &'static str {
        match self {
            CssPropertyType::TextColor => "color",
            CssPropertyType::FontSize => "font-size",
            CssPropertyType::FontFamily => "font-family",
            CssPropertyType::TextAlign => "text-align",
            CssPropertyType::LetterSpacing => "letter-spacing",
            CssPropertyType::LineHeight => "line-height",
            CssPropertyType::WordSpacing => "word-spacing",
            CssPropertyType::TabWidth => "tab-width",
            CssPropertyType::Cursor => "cursor",
            CssPropertyType::Display => "display",
            CssPropertyType::Float => "float",
            CssPropertyType::BoxSizing => "box-sizing",
            CssPropertyType::Width => "width",
            CssPropertyType::Height => "height",
            CssPropertyType::MinWidth => "min-width",
            CssPropertyType::MinHeight => "min-height",
            CssPropertyType::MaxWidth => "max-width",
            CssPropertyType::MaxHeight => "max-height",
            CssPropertyType::Position => "position",
            CssPropertyType::Top => "top",
            CssPropertyType::Right => "right",
            CssPropertyType::Left => "left",
            CssPropertyType::Bottom => "bottom",
            CssPropertyType::FlexWrap => "flex-wrap",
            CssPropertyType::FlexDirection => "flex-direction",
            CssPropertyType::FlexGrow => "flex-grow",
            CssPropertyType::FlexShrink => "flex-shrink",
            CssPropertyType::JustifyContent => "justify-content",
            CssPropertyType::AlignItems => "align-items",
            CssPropertyType::AlignContent => "align-content",
            CssPropertyType::BackgroundContent => "background",
            CssPropertyType::BackgroundPosition => "background-position",
            CssPropertyType::BackgroundSize => "background-size",
            CssPropertyType::BackgroundRepeat => "background-repeat",
            CssPropertyType::OverflowX => "overflow-x",
            CssPropertyType::OverflowY => "overflow-y",
            CssPropertyType::PaddingTop => "padding-top",
            CssPropertyType::PaddingLeft => "padding-left",
            CssPropertyType::PaddingRight => "padding-right",
            CssPropertyType::PaddingBottom => "padding-bottom",
            CssPropertyType::MarginTop => "margin-top",
            CssPropertyType::MarginLeft => "margin-left",
            CssPropertyType::MarginRight => "margin-right",
            CssPropertyType::MarginBottom => "margin-bottom",
            CssPropertyType::BorderTopLeftRadius => "border-top-left-radius",
            CssPropertyType::BorderTopRightRadius => "border-top-right-radius",
            CssPropertyType::BorderBottomLeftRadius => "border-bottom-left-radius",
            CssPropertyType::BorderBottomRightRadius => "border-bottom-right-radius",
            CssPropertyType::BorderTopColor => "border-top-color",
            CssPropertyType::BorderRightColor => "border-right-color",
            CssPropertyType::BorderLeftColor => "border-left-color",
            CssPropertyType::BorderBottomColor => "border-bottom-color",
            CssPropertyType::BorderTopStyle => "border-top-style",
            CssPropertyType::BorderRightStyle => "border-right-style",
            CssPropertyType::BorderLeftStyle => "border-left-style",
            CssPropertyType::BorderBottomStyle => "border-bottom-style",
            CssPropertyType::BorderTopWidth => "border-top-width",
            CssPropertyType::BorderRightWidth => "border-right-width",
            CssPropertyType::BorderLeftWidth => "border-left-width",
            CssPropertyType::BorderBottomWidth => "border-bottom-width",
            CssPropertyType::BoxShadowLeft => "-azul-box-shadow-left",
            CssPropertyType::BoxShadowRight => "-azul-box-shadow-right",
            CssPropertyType::BoxShadowTop => "-azul-box-shadow-top",
            CssPropertyType::BoxShadowBottom => "-azul-box-shadow-bottom",
            CssPropertyType::ScrollbarStyle => "-azul-scrollbar-style",
            CssPropertyType::Opacity => "opacity",
            CssPropertyType::Transform => "transform",
            CssPropertyType::TransformOrigin => "transform-origin",
            CssPropertyType::PerspectiveOrigin => "perspective-origin",
            CssPropertyType::BackfaceVisibility => "backface-visibility",
            CssPropertyType::MixBlendMode => "mix-blend-mode",
            CssPropertyType::Filter => "filter",
            CssPropertyType::BackdropFilter => "backdrop-filter",
            CssPropertyType::TextShadow => "text-shadow",
        }
    }

    /// Returns whether this property will be inherited during cascading
    pub fn is_inheritable(&self) -> bool {
        use self::CssPropertyType::*;
        match self {
            TextColor | FontFamily | FontSize | LineHeight | TextAlign => true,
            _ => false,
        }
    }

    /// Returns whether this property can trigger a re-layout (important for incremental layout and caching layouted DOMs).
    pub fn can_trigger_relayout(&self) -> bool {
        use self::CssPropertyType::*;

        // Since the border can be larger than the content,
        // in which case the content needs to be re-layouted, assume true for Border

        // FontFamily, FontSize, LetterSpacing and LineHeight can affect
        // the text layout and therefore the screen layout

        match self {
            TextColor
            | Cursor
            | BackgroundContent
            | BackgroundPosition
            | BackgroundSize
            | BackgroundRepeat
            | BorderTopLeftRadius
            | BorderTopRightRadius
            | BorderBottomLeftRadius
            | BorderBottomRightRadius
            | BorderTopColor
            | BorderRightColor
            | BorderLeftColor
            | BorderBottomColor
            | BorderTopStyle
            | BorderRightStyle
            | BorderLeftStyle
            | BorderBottomStyle
            | BoxShadowLeft
            | BoxShadowRight
            | BoxShadowTop
            | BoxShadowBottom
            | ScrollbarStyle
            | Opacity
            | Transform
            | TransformOrigin
            | PerspectiveOrigin
            | BackfaceVisibility
            | MixBlendMode
            | Filter
            | BackdropFilter
            | TextShadow => false,
            _ => true,
        }
    }

    /// Returns whether the property is a GPU property (currently only opacity and transforms)
    pub fn is_gpu_only_property(&self) -> bool {
        match self {
            CssPropertyType::Opacity |
            CssPropertyType::Transform /* | CssPropertyType::Color */ => true,
            _ => false
        }
    }
}

impl fmt::Debug for CssPropertyType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl fmt::Display for CssPropertyType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

/// Represents one parsed CSS key-value pair, such as `"width: 20px"` => `CssProperty::Width(LayoutWidth::px(20.0))`
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(C, u8)]
pub enum CssProperty {
    TextColor(StyleTextColorValue),
    FontSize(StyleFontSizeValue),
    FontFamily(StyleFontFamilyVecValue),
    TextAlign(StyleTextAlignValue),
    LetterSpacing(StyleLetterSpacingValue),
    LineHeight(StyleLineHeightValue),
    WordSpacing(StyleWordSpacingValue),
    TabWidth(StyleTabWidthValue),
    Cursor(StyleCursorValue),
    Display(LayoutDisplayValue),
    Float(LayoutFloatValue),
    BoxSizing(LayoutBoxSizingValue),
    Width(LayoutWidthValue),
    Height(LayoutHeightValue),
    MinWidth(LayoutMinWidthValue),
    MinHeight(LayoutMinHeightValue),
    MaxWidth(LayoutMaxWidthValue),
    MaxHeight(LayoutMaxHeightValue),
    Position(LayoutPositionValue),
    Top(LayoutTopValue),
    Right(LayoutRightValue),
    Left(LayoutLeftValue),
    Bottom(LayoutBottomValue),
    FlexWrap(LayoutFlexWrapValue),
    FlexDirection(LayoutFlexDirectionValue),
    FlexGrow(LayoutFlexGrowValue),
    FlexShrink(LayoutFlexShrinkValue),
    JustifyContent(LayoutJustifyContentValue),
    AlignItems(LayoutAlignItemsValue),
    AlignContent(LayoutAlignContentValue),
    BackgroundContent(StyleBackgroundContentVecValue),
    BackgroundPosition(StyleBackgroundPositionVecValue),
    BackgroundSize(StyleBackgroundSizeVecValue),
    BackgroundRepeat(StyleBackgroundRepeatVecValue),
    OverflowX(LayoutOverflowValue),
    OverflowY(LayoutOverflowValue),
    PaddingTop(LayoutPaddingTopValue),
    PaddingLeft(LayoutPaddingLeftValue),
    PaddingRight(LayoutPaddingRightValue),
    PaddingBottom(LayoutPaddingBottomValue),
    MarginTop(LayoutMarginTopValue),
    MarginLeft(LayoutMarginLeftValue),
    MarginRight(LayoutMarginRightValue),
    MarginBottom(LayoutMarginBottomValue),
    BorderTopLeftRadius(StyleBorderTopLeftRadiusValue),
    BorderTopRightRadius(StyleBorderTopRightRadiusValue),
    BorderBottomLeftRadius(StyleBorderBottomLeftRadiusValue),
    BorderBottomRightRadius(StyleBorderBottomRightRadiusValue),
    BorderTopColor(StyleBorderTopColorValue),
    BorderRightColor(StyleBorderRightColorValue),
    BorderLeftColor(StyleBorderLeftColorValue),
    BorderBottomColor(StyleBorderBottomColorValue),
    BorderTopStyle(StyleBorderTopStyleValue),
    BorderRightStyle(StyleBorderRightStyleValue),
    BorderLeftStyle(StyleBorderLeftStyleValue),
    BorderBottomStyle(StyleBorderBottomStyleValue),
    BorderTopWidth(LayoutBorderTopWidthValue),
    BorderRightWidth(LayoutBorderRightWidthValue),
    BorderLeftWidth(LayoutBorderLeftWidthValue),
    BorderBottomWidth(LayoutBorderBottomWidthValue),
    BoxShadowLeft(StyleBoxShadowValue),
    BoxShadowRight(StyleBoxShadowValue),
    BoxShadowTop(StyleBoxShadowValue),
    BoxShadowBottom(StyleBoxShadowValue),
    ScrollbarStyle(ScrollbarStyleValue),
    Opacity(StyleOpacityValue),
    Transform(StyleTransformVecValue),
    TransformOrigin(StyleTransformOriginValue),
    PerspectiveOrigin(StylePerspectiveOriginValue),
    BackfaceVisibility(StyleBackfaceVisibilityValue),
    MixBlendMode(StyleMixBlendModeValue),
    Filter(StyleFilterVecValue),
    BackdropFilter(StyleFilterVecValue),
    TextShadow(StyleBoxShadowValue),
}

macro_rules! css_property_from_type {
    ($prop_type:expr, $content_type:ident) => {{
        match $prop_type {
            CssPropertyType::TextColor => {
                CssProperty::TextColor(StyleTextColorValue::$content_type)
            }
            CssPropertyType::FontSize => CssProperty::FontSize(StyleFontSizeValue::$content_type),
            CssPropertyType::FontFamily => {
                CssProperty::FontFamily(StyleFontFamilyVecValue::$content_type)
            }
            CssPropertyType::TextAlign => {
                CssProperty::TextAlign(StyleTextAlignValue::$content_type)
            }
            CssPropertyType::LetterSpacing => {
                CssProperty::LetterSpacing(StyleLetterSpacingValue::$content_type)
            }
            CssPropertyType::LineHeight => {
                CssProperty::LineHeight(StyleLineHeightValue::$content_type)
            }
            CssPropertyType::WordSpacing => {
                CssProperty::WordSpacing(StyleWordSpacingValue::$content_type)
            }
            CssPropertyType::TabWidth => CssProperty::TabWidth(StyleTabWidthValue::$content_type),
            CssPropertyType::Cursor => CssProperty::Cursor(StyleCursorValue::$content_type),
            CssPropertyType::Display => CssProperty::Display(LayoutDisplayValue::$content_type),
            CssPropertyType::Float => CssProperty::Float(LayoutFloatValue::$content_type),
            CssPropertyType::BoxSizing => {
                CssProperty::BoxSizing(LayoutBoxSizingValue::$content_type)
            }
            CssPropertyType::Width => CssProperty::Width(LayoutWidthValue::$content_type),
            CssPropertyType::Height => CssProperty::Height(LayoutHeightValue::$content_type),
            CssPropertyType::MinWidth => CssProperty::MinWidth(LayoutMinWidthValue::$content_type),
            CssPropertyType::MinHeight => {
                CssProperty::MinHeight(LayoutMinHeightValue::$content_type)
            }
            CssPropertyType::MaxWidth => CssProperty::MaxWidth(LayoutMaxWidthValue::$content_type),
            CssPropertyType::MaxHeight => {
                CssProperty::MaxHeight(LayoutMaxHeightValue::$content_type)
            }
            CssPropertyType::Position => CssProperty::Position(LayoutPositionValue::$content_type),
            CssPropertyType::Top => CssProperty::Top(LayoutTopValue::$content_type),
            CssPropertyType::Right => CssProperty::Right(LayoutRightValue::$content_type),
            CssPropertyType::Left => CssProperty::Left(LayoutLeftValue::$content_type),
            CssPropertyType::Bottom => CssProperty::Bottom(LayoutBottomValue::$content_type),
            CssPropertyType::FlexWrap => CssProperty::FlexWrap(LayoutFlexWrapValue::$content_type),
            CssPropertyType::FlexDirection => {
                CssProperty::FlexDirection(LayoutFlexDirectionValue::$content_type)
            }
            CssPropertyType::FlexGrow => CssProperty::FlexGrow(LayoutFlexGrowValue::$content_type),
            CssPropertyType::FlexShrink => {
                CssProperty::FlexShrink(LayoutFlexShrinkValue::$content_type)
            }
            CssPropertyType::JustifyContent => {
                CssProperty::JustifyContent(LayoutJustifyContentValue::$content_type)
            }
            CssPropertyType::AlignItems => {
                CssProperty::AlignItems(LayoutAlignItemsValue::$content_type)
            }
            CssPropertyType::AlignContent => {
                CssProperty::AlignContent(LayoutAlignContentValue::$content_type)
            }
            CssPropertyType::BackgroundContent => {
                CssProperty::BackgroundContent(StyleBackgroundContentVecValue::$content_type)
            }
            CssPropertyType::BackgroundPosition => {
                CssProperty::BackgroundPosition(StyleBackgroundPositionVecValue::$content_type)
            }
            CssPropertyType::BackgroundSize => {
                CssProperty::BackgroundSize(StyleBackgroundSizeVecValue::$content_type)
            }
            CssPropertyType::BackgroundRepeat => {
                CssProperty::BackgroundRepeat(StyleBackgroundRepeatVecValue::$content_type)
            }
            CssPropertyType::OverflowX => {
                CssProperty::OverflowX(LayoutOverflowValue::$content_type)
            }
            CssPropertyType::OverflowY => {
                CssProperty::OverflowY(LayoutOverflowValue::$content_type)
            }
            CssPropertyType::PaddingTop => {
                CssProperty::PaddingTop(LayoutPaddingTopValue::$content_type)
            }
            CssPropertyType::PaddingLeft => {
                CssProperty::PaddingLeft(LayoutPaddingLeftValue::$content_type)
            }
            CssPropertyType::PaddingRight => {
                CssProperty::PaddingRight(LayoutPaddingRightValue::$content_type)
            }
            CssPropertyType::PaddingBottom => {
                CssProperty::PaddingBottom(LayoutPaddingBottomValue::$content_type)
            }
            CssPropertyType::MarginTop => {
                CssProperty::MarginTop(LayoutMarginTopValue::$content_type)
            }
            CssPropertyType::MarginLeft => {
                CssProperty::MarginLeft(LayoutMarginLeftValue::$content_type)
            }
            CssPropertyType::MarginRight => {
                CssProperty::MarginRight(LayoutMarginRightValue::$content_type)
            }
            CssPropertyType::MarginBottom => {
                CssProperty::MarginBottom(LayoutMarginBottomValue::$content_type)
            }
            CssPropertyType::BorderTopLeftRadius => {
                CssProperty::BorderTopLeftRadius(StyleBorderTopLeftRadiusValue::$content_type)
            }
            CssPropertyType::BorderTopRightRadius => {
                CssProperty::BorderTopRightRadius(StyleBorderTopRightRadiusValue::$content_type)
            }
            CssPropertyType::BorderBottomLeftRadius => {
                CssProperty::BorderBottomLeftRadius(StyleBorderBottomLeftRadiusValue::$content_type)
            }
            CssPropertyType::BorderBottomRightRadius => CssProperty::BorderBottomRightRadius(
                StyleBorderBottomRightRadiusValue::$content_type,
            ),
            CssPropertyType::BorderTopColor => {
                CssProperty::BorderTopColor(StyleBorderTopColorValue::$content_type)
            }
            CssPropertyType::BorderRightColor => {
                CssProperty::BorderRightColor(StyleBorderRightColorValue::$content_type)
            }
            CssPropertyType::BorderLeftColor => {
                CssProperty::BorderLeftColor(StyleBorderLeftColorValue::$content_type)
            }
            CssPropertyType::BorderBottomColor => {
                CssProperty::BorderBottomColor(StyleBorderBottomColorValue::$content_type)
            }
            CssPropertyType::BorderTopStyle => {
                CssProperty::BorderTopStyle(StyleBorderTopStyleValue::$content_type)
            }
            CssPropertyType::BorderRightStyle => {
                CssProperty::BorderRightStyle(StyleBorderRightStyleValue::$content_type)
            }
            CssPropertyType::BorderLeftStyle => {
                CssProperty::BorderLeftStyle(StyleBorderLeftStyleValue::$content_type)
            }
            CssPropertyType::BorderBottomStyle => {
                CssProperty::BorderBottomStyle(StyleBorderBottomStyleValue::$content_type)
            }
            CssPropertyType::BorderTopWidth => {
                CssProperty::BorderTopWidth(LayoutBorderTopWidthValue::$content_type)
            }
            CssPropertyType::BorderRightWidth => {
                CssProperty::BorderRightWidth(LayoutBorderRightWidthValue::$content_type)
            }
            CssPropertyType::BorderLeftWidth => {
                CssProperty::BorderLeftWidth(LayoutBorderLeftWidthValue::$content_type)
            }
            CssPropertyType::BorderBottomWidth => {
                CssProperty::BorderBottomWidth(LayoutBorderBottomWidthValue::$content_type)
            }
            CssPropertyType::BoxShadowLeft => {
                CssProperty::BoxShadowLeft(StyleBoxShadowValue::$content_type)
            }
            CssPropertyType::BoxShadowRight => {
                CssProperty::BoxShadowRight(StyleBoxShadowValue::$content_type)
            }
            CssPropertyType::BoxShadowTop => {
                CssProperty::BoxShadowTop(StyleBoxShadowValue::$content_type)
            }
            CssPropertyType::BoxShadowBottom => {
                CssProperty::BoxShadowBottom(StyleBoxShadowValue::$content_type)
            }
            CssPropertyType::ScrollbarStyle => {
                CssProperty::ScrollbarStyle(ScrollbarStyleValue::$content_type)
            }
            CssPropertyType::Opacity => CssProperty::Opacity(StyleOpacityValue::$content_type),
            CssPropertyType::Transform => {
                CssProperty::Transform(StyleTransformVecValue::$content_type)
            }
            CssPropertyType::PerspectiveOrigin => {
                CssProperty::PerspectiveOrigin(StylePerspectiveOriginValue::$content_type)
            }
            CssPropertyType::TransformOrigin => {
                CssProperty::TransformOrigin(StyleTransformOriginValue::$content_type)
            }
            CssPropertyType::BackfaceVisibility => {
                CssProperty::BackfaceVisibility(StyleBackfaceVisibilityValue::$content_type)
            }
            CssPropertyType::MixBlendMode => {
                CssProperty::MixBlendMode(StyleMixBlendModeValue::$content_type)
            }
            CssPropertyType::Filter => CssProperty::Filter(StyleFilterVecValue::$content_type),
            CssPropertyType::BackdropFilter => {
                CssProperty::BackdropFilter(StyleFilterVecValue::$content_type)
            }
            CssPropertyType::TextShadow => {
                CssProperty::TextShadow(StyleBoxShadowValue::$content_type)
            }
        }
    }};
}

impl CssProperty {
    pub const fn const_text_color(input: StyleTextColor) -> Self {
        CssProperty::TextColor(StyleTextColorValue::Exact(input))
    }
    pub const fn const_font_size(input: StyleFontSize) -> Self {
        CssProperty::FontSize(StyleFontSizeValue::Exact(input))
    }
    pub const fn const_font_family(input: Vec<StyleFontFamily>) -> Self {
        CssProperty::FontFamily(StyleFontFamilyVecValue::Exact(input))
    }
    pub const fn const_text_align(input: StyleTextAlign) -> Self {
        CssProperty::TextAlign(StyleTextAlignValue::Exact(input))
    }
    pub const fn const_letter_spacing(input: StyleLetterSpacing) -> Self {
        CssProperty::LetterSpacing(StyleLetterSpacingValue::Exact(input))
    }
    pub const fn const_line_height(input: StyleLineHeight) -> Self {
        CssProperty::LineHeight(StyleLineHeightValue::Exact(input))
    }
    pub const fn const_word_spacing(input: StyleWordSpacing) -> Self {
        CssProperty::WordSpacing(StyleWordSpacingValue::Exact(input))
    }
    pub const fn const_tab_width(input: StyleTabWidth) -> Self {
        CssProperty::TabWidth(StyleTabWidthValue::Exact(input))
    }
    pub const fn const_cursor(input: StyleCursor) -> Self {
        CssProperty::Cursor(StyleCursorValue::Exact(input))
    }
    pub const fn const_display(input: LayoutDisplay) -> Self {
        CssProperty::Display(LayoutDisplayValue::Exact(input))
    }
    pub const fn const_float(input: LayoutFloat) -> Self {
        CssProperty::Float(LayoutFloatValue::Exact(input))
    }
    pub const fn const_box_sizing(input: LayoutBoxSizing) -> Self {
        CssProperty::BoxSizing(LayoutBoxSizingValue::Exact(input))
    }
    pub const fn const_width(input: LayoutWidth) -> Self {
        CssProperty::Width(LayoutWidthValue::Exact(input))
    }
    pub const fn const_height(input: LayoutHeight) -> Self {
        CssProperty::Height(LayoutHeightValue::Exact(input))
    }
    pub const fn const_min_width(input: LayoutMinWidth) -> Self {
        CssProperty::MinWidth(LayoutMinWidthValue::Exact(input))
    }
    pub const fn const_min_height(input: LayoutMinHeight) -> Self {
        CssProperty::MinHeight(LayoutMinHeightValue::Exact(input))
    }
    pub const fn const_max_width(input: LayoutMaxWidth) -> Self {
        CssProperty::MaxWidth(LayoutMaxWidthValue::Exact(input))
    }
    pub const fn const_max_height(input: LayoutMaxHeight) -> Self {
        CssProperty::MaxHeight(LayoutMaxHeightValue::Exact(input))
    }
    pub const fn const_position(input: LayoutPosition) -> Self {
        CssProperty::Position(LayoutPositionValue::Exact(input))
    }
    pub const fn const_top(input: LayoutTop) -> Self {
        CssProperty::Top(LayoutTopValue::Exact(input))
    }
    pub const fn const_right(input: LayoutRight) -> Self {
        CssProperty::Right(LayoutRightValue::Exact(input))
    }
    pub const fn const_left(input: LayoutLeft) -> Self {
        CssProperty::Left(LayoutLeftValue::Exact(input))
    }
    pub const fn const_bottom(input: LayoutBottom) -> Self {
        CssProperty::Bottom(LayoutBottomValue::Exact(input))
    }
    pub const fn const_flex_wrap(input: LayoutFlexWrap) -> Self {
        CssProperty::FlexWrap(LayoutFlexWrapValue::Exact(input))
    }
    pub const fn const_flex_direction(input: LayoutFlexDirection) -> Self {
        CssProperty::FlexDirection(LayoutFlexDirectionValue::Exact(input))
    }
    pub const fn const_flex_grow(input: LayoutFlexGrow) -> Self {
        CssProperty::FlexGrow(LayoutFlexGrowValue::Exact(input))
    }
    pub const fn const_flex_shrink(input: LayoutFlexShrink) -> Self {
        CssProperty::FlexShrink(LayoutFlexShrinkValue::Exact(input))
    }
    pub const fn const_justify_content(input: LayoutJustifyContent) -> Self {
        CssProperty::JustifyContent(LayoutJustifyContentValue::Exact(input))
    }
    pub const fn const_align_items(input: LayoutAlignItems) -> Self {
        CssProperty::AlignItems(LayoutAlignItemsValue::Exact(input))
    }
    pub const fn const_align_content(input: LayoutAlignContent) -> Self {
        CssProperty::AlignContent(LayoutAlignContentValue::Exact(input))
    }
    pub const fn const_background_content(input: Vec<StyleBackgroundContent>) -> Self {
        CssProperty::BackgroundContent(StyleBackgroundContentVecValue::Exact(input))
    }
    pub const fn const_background_position(input: Vec<StyleBackgroundPosition>) -> Self {
        CssProperty::BackgroundPosition(StyleBackgroundPositionVecValue::Exact(input))
    }
    pub const fn const_background_size(input: Vec<StyleBackgroundSize>) -> Self {
        CssProperty::BackgroundSize(StyleBackgroundSizeVecValue::Exact(input))
    }
    pub const fn const_background_repeat(input: Vec<StyleBackgroundRepeat>) -> Self {
        CssProperty::BackgroundRepeat(StyleBackgroundRepeatVecValue::Exact(input))
    }
    pub const fn const_overflow_x(input: LayoutOverflow) -> Self {
        CssProperty::OverflowX(LayoutOverflowValue::Exact(input))
    }
    pub const fn const_overflow_y(input: LayoutOverflow) -> Self {
        CssProperty::OverflowY(LayoutOverflowValue::Exact(input))
    }
    pub const fn const_padding_top(input: LayoutPaddingTop) -> Self {
        CssProperty::PaddingTop(LayoutPaddingTopValue::Exact(input))
    }
    pub const fn const_padding_left(input: LayoutPaddingLeft) -> Self {
        CssProperty::PaddingLeft(LayoutPaddingLeftValue::Exact(input))
    }
    pub const fn const_padding_right(input: LayoutPaddingRight) -> Self {
        CssProperty::PaddingRight(LayoutPaddingRightValue::Exact(input))
    }
    pub const fn const_padding_bottom(input: LayoutPaddingBottom) -> Self {
        CssProperty::PaddingBottom(LayoutPaddingBottomValue::Exact(input))
    }
    pub const fn const_margin_top(input: LayoutMarginTop) -> Self {
        CssProperty::MarginTop(LayoutMarginTopValue::Exact(input))
    }
    pub const fn const_margin_left(input: LayoutMarginLeft) -> Self {
        CssProperty::MarginLeft(LayoutMarginLeftValue::Exact(input))
    }
    pub const fn const_margin_right(input: LayoutMarginRight) -> Self {
        CssProperty::MarginRight(LayoutMarginRightValue::Exact(input))
    }
    pub const fn const_margin_bottom(input: LayoutMarginBottom) -> Self {
        CssProperty::MarginBottom(LayoutMarginBottomValue::Exact(input))
    }
    pub const fn const_border_top_left_radius(input: StyleBorderTopLeftRadius) -> Self {
        CssProperty::BorderTopLeftRadius(StyleBorderTopLeftRadiusValue::Exact(input))
    }
    pub const fn const_border_top_right_radius(input: StyleBorderTopRightRadius) -> Self {
        CssProperty::BorderTopRightRadius(StyleBorderTopRightRadiusValue::Exact(input))
    }
    pub const fn const_border_bottom_left_radius(input: StyleBorderBottomLeftRadius) -> Self {
        CssProperty::BorderBottomLeftRadius(StyleBorderBottomLeftRadiusValue::Exact(input))
    }
    pub const fn const_border_bottom_right_radius(input: StyleBorderBottomRightRadius) -> Self {
        CssProperty::BorderBottomRightRadius(StyleBorderBottomRightRadiusValue::Exact(input))
    }
    pub const fn const_border_top_color(input: StyleBorderTopColor) -> Self {
        CssProperty::BorderTopColor(StyleBorderTopColorValue::Exact(input))
    }
    pub const fn const_border_right_color(input: StyleBorderRightColor) -> Self {
        CssProperty::BorderRightColor(StyleBorderRightColorValue::Exact(input))
    }
    pub const fn const_border_left_color(input: StyleBorderLeftColor) -> Self {
        CssProperty::BorderLeftColor(StyleBorderLeftColorValue::Exact(input))
    }
    pub const fn const_border_bottom_color(input: StyleBorderBottomColor) -> Self {
        CssProperty::BorderBottomColor(StyleBorderBottomColorValue::Exact(input))
    }
    pub const fn const_border_top_style(input: StyleBorderTopStyle) -> Self {
        CssProperty::BorderTopStyle(StyleBorderTopStyleValue::Exact(input))
    }
    pub const fn const_border_right_style(input: StyleBorderRightStyle) -> Self {
        CssProperty::BorderRightStyle(StyleBorderRightStyleValue::Exact(input))
    }
    pub const fn const_border_left_style(input: StyleBorderLeftStyle) -> Self {
        CssProperty::BorderLeftStyle(StyleBorderLeftStyleValue::Exact(input))
    }
    pub const fn const_border_bottom_style(input: StyleBorderBottomStyle) -> Self {
        CssProperty::BorderBottomStyle(StyleBorderBottomStyleValue::Exact(input))
    }
    pub const fn const_border_top_width(input: LayoutBorderTopWidth) -> Self {
        CssProperty::BorderTopWidth(LayoutBorderTopWidthValue::Exact(input))
    }
    pub const fn const_border_right_width(input: LayoutBorderRightWidth) -> Self {
        CssProperty::BorderRightWidth(LayoutBorderRightWidthValue::Exact(input))
    }
    pub const fn const_border_left_width(input: LayoutBorderLeftWidth) -> Self {
        CssProperty::BorderLeftWidth(LayoutBorderLeftWidthValue::Exact(input))
    }
    pub const fn const_border_bottom_width(input: LayoutBorderBottomWidth) -> Self {
        CssProperty::BorderBottomWidth(LayoutBorderBottomWidthValue::Exact(input))
    }
    pub const fn const_box_shadow_left(input: StyleBoxShadow) -> Self {
        CssProperty::BoxShadowLeft(StyleBoxShadowValue::Exact(input))
    }
    pub const fn const_box_shadow_right(input: StyleBoxShadow) -> Self {
        CssProperty::BoxShadowRight(StyleBoxShadowValue::Exact(input))
    }
    pub const fn const_box_shadow_top(input: StyleBoxShadow) -> Self {
        CssProperty::BoxShadowTop(StyleBoxShadowValue::Exact(input))
    }
    pub const fn const_box_shadow_bottom(input: StyleBoxShadow) -> Self {
        CssProperty::BoxShadowBottom(StyleBoxShadowValue::Exact(input))
    }
    pub const fn const_opacity(input: StyleOpacity) -> Self {
        CssProperty::Opacity(StyleOpacityValue::Exact(input))
    }
    pub const fn const_transform(input: Vec<StyleTransform>) -> Self {
        CssProperty::Transform(StyleTransformVecValue::Exact(input))
    }
    pub const fn const_transform_origin(input: StyleTransformOrigin) -> Self {
        CssProperty::TransformOrigin(StyleTransformOriginValue::Exact(input))
    }
    pub const fn const_perspective_origin(input: StylePerspectiveOrigin) -> Self {
        CssProperty::PerspectiveOrigin(StylePerspectiveOriginValue::Exact(input))
    }
    pub const fn const_backface_visiblity(input: StyleBackfaceVisibility) -> Self {
        CssProperty::BackfaceVisibility(StyleBackfaceVisibilityValue::Exact(input))
    }
}
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C, u8)]
pub enum AnimationInterpolationFunction {
    Ease,
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier(SvgCubicCurve),
}

#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd)]
#[repr(C)]
pub struct SvgPoint {
    pub x: f32,
    pub y: f32,
}

impl SvgPoint {
    #[inline]
    pub fn distance(&self, other: Self) -> f64 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        libm::hypotf(dx, dy) as f64
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd)]
#[repr(C)]
pub struct SvgRect {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub radius_top_left: f32,
    pub radius_top_right: f32,
    pub radius_bottom_left: f32,
    pub radius_bottom_right: f32,
}

impl SvgRect {
    pub fn union_with(&mut self, other: &Self) {
        let self_max_x = self.x + self.width;
        let self_max_y = self.y + self.height;
        let self_min_x = self.x;
        let self_min_y = self.y;

        let other_max_x = other.x + other.width;
        let other_max_y = other.y + other.height;
        let other_min_x = other.x;
        let other_min_y = other.y;

        let max_x = self_max_x.max(other_max_x);
        let max_y = self_max_y.max(other_max_y);
        let min_x = self_min_x.min(other_min_x);
        let min_y = self_min_y.min(other_min_y);

        self.x = min_x;
        self.y = min_y;
        self.width = max_x - min_x;
        self.height = max_y - min_y;
    }

    /// Note: does not incorporate rounded edges!
    /// Origin of x and y is assumed to be the top left corner
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        x > self.x && x < self.x + self.width && y > self.y && y < self.y + self.height
    }

    pub fn get_center(&self) -> SvgPoint {
        SvgPoint {
            x: self.x + (self.width / 2.0),
            y: self.y + (self.height / 2.0),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[repr(C)]
pub struct SvgCubicCurve {
    pub start: SvgPoint,
    pub ctrl_1: SvgPoint,
    pub ctrl_2: SvgPoint,
    pub end: SvgPoint,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[repr(C)]
pub struct SvgVector {
    pub x: f64,
    pub y: f64,
}

impl SvgVector {
    /// Returns the angle of the vector in degrees
    #[inline]
    pub fn angle_degrees(&self) -> f64 {
        //  y
        //  |  /
        //  | /
        //   / a)
        //   ___ x

        (-self.x).atan2(self.y).to_degrees()
    }

    #[inline]
    #[must_use = "returns a new vector"]
    pub fn normalize(&self) -> Self {
        let tangent_length = libm::hypotf(self.x as f32, self.y as f32) as f64;

        Self {
            x: self.x / tangent_length,
            y: self.y / tangent_length,
        }
    }

    /// Rotate the vector 90 degrees counter-clockwise
    #[must_use = "returns a new vector"]
    #[inline]
    pub fn rotate_90deg_ccw(&self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }
}

const STEP_SIZE: usize = 20;
const STEP_SIZE_F64: f64 = 0.05;

impl SvgCubicCurve {
    pub fn reverse(&mut self) {
        let mut temp = self.start;
        self.start = self.end;
        self.end = temp;
        temp = self.ctrl_1;
        self.ctrl_1 = self.ctrl_2;
        self.ctrl_2 = temp;
    }

    pub fn get_start(&self) -> SvgPoint {
        self.start
    }
    pub fn get_end(&self) -> SvgPoint {
        self.end
    }

    // evaluate the curve at t
    pub fn get_x_at_t(&self, t: f64) -> f64 {
        let c_x = 3.0 * (self.ctrl_1.x as f64 - self.start.x as f64);
        let b_x = 3.0 * (self.ctrl_2.x as f64 - self.ctrl_1.x as f64) - c_x;
        let a_x = self.end.x as f64 - self.start.x as f64 - c_x - b_x;

        (a_x * t * t * t) + (b_x * t * t) + (c_x * t) + self.start.x as f64
    }

    pub fn get_y_at_t(&self, t: f64) -> f64 {
        let c_x = 3.0 * (self.ctrl_1.y as f64 - self.start.y as f64);
        let b_x = 3.0 * (self.ctrl_2.y as f64 - self.ctrl_1.y as f64) - c_x;
        let a_x = self.end.y as f64 - self.start.y as f64 - c_x - b_x;

        (a_x * t * t * t) + (b_x * t * t) + (c_x * t) + self.start.y as f64
    }

    pub fn get_length(&self) -> f64 {
        // NOTE: this arc length parametrization is not very precise, but fast
        let mut arc_length = 0.0;
        let mut prev_point = self.get_start();

        for i in 0..STEP_SIZE {
            let t_next = (i + 1) as f64 * STEP_SIZE_F64;
            let next_point = SvgPoint {
                x: self.get_x_at_t(t_next) as f32,
                y: self.get_y_at_t(t_next) as f32,
            };
            arc_length += prev_point.distance(next_point);
            prev_point = next_point;
        }

        arc_length
    }

    pub fn get_t_at_offset(&self, offset: f64) -> f64 {
        // step through the line until the offset is reached,
        // then interpolate linearly between the
        // current at the last sampled point
        let mut arc_length = 0.0;
        let mut t_current = 0.0;
        let mut prev_point = self.get_start();

        for i in 0..STEP_SIZE {
            let t_next = (i + 1) as f64 * STEP_SIZE_F64;
            let next_point = SvgPoint {
                x: self.get_x_at_t(t_next) as f32,
                y: self.get_y_at_t(t_next) as f32,
            };

            let distance = prev_point.distance(next_point);

            arc_length += distance;

            // linearly interpolate between last t and current t
            if arc_length > offset {
                let remaining = arc_length - offset;
                return t_current + (remaining / distance) * STEP_SIZE_F64;
            }

            prev_point = next_point;
            t_current = t_next;
        }

        t_current
    }

    pub fn get_bounds(&self) -> SvgRect {
        let min_x = self
            .start
            .x
            .min(self.end.x)
            .min(self.ctrl_1.x)
            .min(self.ctrl_2.x);
        let max_x = self
            .start
            .x
            .max(self.end.x)
            .max(self.ctrl_1.x)
            .max(self.ctrl_2.x);

        let min_y = self
            .start
            .y
            .min(self.end.y)
            .min(self.ctrl_1.y)
            .min(self.ctrl_2.y);
        let max_y = self
            .start
            .y
            .max(self.end.y)
            .max(self.ctrl_1.y)
            .max(self.ctrl_2.y);

        let width = (max_x - min_x).abs();
        let height = (max_y - min_y).abs();

        SvgRect {
            width,
            height,
            x: min_x,
            y: min_y,
            ..SvgRect::default()
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[repr(C)]
pub struct SvgQuadraticCurve {
    pub start: SvgPoint,
    pub ctrl: SvgPoint,
    pub end: SvgPoint,
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct InterpolateResolver {
    pub interpolate_func: AnimationInterpolationFunction,
    pub parent_rect_width: f32,
    pub parent_rect_height: f32,
    pub current_rect_width: f32,
    pub current_rect_height: f32,
}

impl CssProperty {}

macro_rules! impl_from_css_prop {
    ($a:ty, $b:ident::$enum_type:ident) => {
        impl From<$a> for $b {
            fn from(e: $a) -> Self {
                $b::$enum_type(CssPropertyValue::from(e))
            }
        }
    };
}

impl_from_css_prop!(StyleTextColor, CssProperty::TextColor);
impl_from_css_prop!(StyleFontSize, CssProperty::FontSize);
impl_from_css_prop!(Vec<StyleFontFamily>, CssProperty::FontFamily);
impl_from_css_prop!(StyleTextAlign, CssProperty::TextAlign);
impl_from_css_prop!(StyleLetterSpacing, CssProperty::LetterSpacing);
impl_from_css_prop!(StyleLineHeight, CssProperty::LineHeight);
impl_from_css_prop!(StyleWordSpacing, CssProperty::WordSpacing);
impl_from_css_prop!(StyleTabWidth, CssProperty::TabWidth);
impl_from_css_prop!(StyleCursor, CssProperty::Cursor);
impl_from_css_prop!(LayoutDisplay, CssProperty::Display);
impl_from_css_prop!(LayoutFloat, CssProperty::Float);
impl_from_css_prop!(LayoutBoxSizing, CssProperty::BoxSizing);
impl_from_css_prop!(LayoutWidth, CssProperty::Width);
impl_from_css_prop!(LayoutHeight, CssProperty::Height);
impl_from_css_prop!(LayoutMinWidth, CssProperty::MinWidth);
impl_from_css_prop!(LayoutMinHeight, CssProperty::MinHeight);
impl_from_css_prop!(LayoutMaxWidth, CssProperty::MaxWidth);
impl_from_css_prop!(LayoutMaxHeight, CssProperty::MaxHeight);
impl_from_css_prop!(LayoutPosition, CssProperty::Position);
impl_from_css_prop!(LayoutTop, CssProperty::Top);
impl_from_css_prop!(LayoutRight, CssProperty::Right);
impl_from_css_prop!(LayoutLeft, CssProperty::Left);
impl_from_css_prop!(LayoutBottom, CssProperty::Bottom);
impl_from_css_prop!(LayoutFlexWrap, CssProperty::FlexWrap);
impl_from_css_prop!(LayoutFlexDirection, CssProperty::FlexDirection);
impl_from_css_prop!(LayoutFlexGrow, CssProperty::FlexGrow);
impl_from_css_prop!(LayoutFlexShrink, CssProperty::FlexShrink);
impl_from_css_prop!(LayoutJustifyContent, CssProperty::JustifyContent);
impl_from_css_prop!(LayoutAlignItems, CssProperty::AlignItems);
impl_from_css_prop!(LayoutAlignContent, CssProperty::AlignContent);
impl_from_css_prop!(Vec<StyleBackgroundContent>, CssProperty::BackgroundContent);
impl_from_css_prop!(
    Vec<StyleBackgroundPosition>,
    CssProperty::BackgroundPosition
);
impl_from_css_prop!(Vec<StyleBackgroundSize>, CssProperty::BackgroundSize);
impl_from_css_prop!(Vec<StyleBackgroundRepeat>, CssProperty::BackgroundRepeat);
impl_from_css_prop!(LayoutPaddingTop, CssProperty::PaddingTop);
impl_from_css_prop!(LayoutPaddingLeft, CssProperty::PaddingLeft);
impl_from_css_prop!(LayoutPaddingRight, CssProperty::PaddingRight);
impl_from_css_prop!(LayoutPaddingBottom, CssProperty::PaddingBottom);
impl_from_css_prop!(LayoutMarginTop, CssProperty::MarginTop);
impl_from_css_prop!(LayoutMarginLeft, CssProperty::MarginLeft);
impl_from_css_prop!(LayoutMarginRight, CssProperty::MarginRight);
impl_from_css_prop!(LayoutMarginBottom, CssProperty::MarginBottom);
impl_from_css_prop!(StyleBorderTopLeftRadius, CssProperty::BorderTopLeftRadius);
impl_from_css_prop!(StyleBorderTopRightRadius, CssProperty::BorderTopRightRadius);
impl_from_css_prop!(
    StyleBorderBottomLeftRadius,
    CssProperty::BorderBottomLeftRadius
);
impl_from_css_prop!(
    StyleBorderBottomRightRadius,
    CssProperty::BorderBottomRightRadius
);
impl_from_css_prop!(StyleBorderTopColor, CssProperty::BorderTopColor);
impl_from_css_prop!(StyleBorderRightColor, CssProperty::BorderRightColor);
impl_from_css_prop!(StyleBorderLeftColor, CssProperty::BorderLeftColor);
impl_from_css_prop!(StyleBorderBottomColor, CssProperty::BorderBottomColor);
impl_from_css_prop!(StyleBorderTopStyle, CssProperty::BorderTopStyle);
impl_from_css_prop!(StyleBorderRightStyle, CssProperty::BorderRightStyle);
impl_from_css_prop!(StyleBorderLeftStyle, CssProperty::BorderLeftStyle);
impl_from_css_prop!(StyleBorderBottomStyle, CssProperty::BorderBottomStyle);
impl_from_css_prop!(LayoutBorderTopWidth, CssProperty::BorderTopWidth);
impl_from_css_prop!(LayoutBorderRightWidth, CssProperty::BorderRightWidth);
impl_from_css_prop!(LayoutBorderLeftWidth, CssProperty::BorderLeftWidth);
impl_from_css_prop!(LayoutBorderBottomWidth, CssProperty::BorderBottomWidth);
impl_from_css_prop!(ScrollbarStyle, CssProperty::ScrollbarStyle);
impl_from_css_prop!(StyleOpacity, CssProperty::Opacity);
impl_from_css_prop!(Vec<StyleTransform>, CssProperty::Transform);
impl_from_css_prop!(StyleTransformOrigin, CssProperty::TransformOrigin);
impl_from_css_prop!(StylePerspectiveOrigin, CssProperty::PerspectiveOrigin);
impl_from_css_prop!(StyleBackfaceVisibility, CssProperty::BackfaceVisibility);
impl_from_css_prop!(StyleMixBlendMode, CssProperty::MixBlendMode);

/// Multiplier for floating point accuracy. Elements such as px or %
/// are only accurate until a certain number of decimal points, therefore
/// they have to be casted to isizes in order to make the f32 values
/// hash-able: Css has a relatively low precision here, roughly 5 digits, i.e
/// `1.00001 == 1.0`
const FP_PRECISION_MULTIPLIER: f32 = 1000.0;
const FP_PRECISION_MULTIPLIER_CONST: isize = FP_PRECISION_MULTIPLIER as isize;

/// Same as PixelValue, but doesn't allow a "%" sign
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct PixelValueNoPercent {
    pub inner: PixelValue,
}

impl fmt::Display for PixelValueNoPercent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl ::core::fmt::Debug for PixelValueNoPercent {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "{}", self)
    }
}

impl PixelValueNoPercent {
    pub fn to_pixels(&self) -> f32 {
        self.inner.to_pixels(0.0)
    }
}

/// FloatValue, but associated with a certain metric (i.e. px, em, etc.)
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct AngleValue {
    pub metric: AngleMetric,
    pub number: FloatValue,
}

impl fmt::Debug for AngleValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

// Manual Debug implementation, because the auto-generated one is nearly unreadable
impl fmt::Display for AngleValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.number, self.metric)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum AngleMetric {
    Degree,
    Radians,
    Grad,
    Turn,
    Percent,
}

impl Default for AngleMetric {
    fn default() -> AngleMetric {
        AngleMetric::Degree
    }
}

impl fmt::Display for AngleMetric {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::AngleMetric::*;
        match self {
            Degree => write!(f, "deg"),
            Radians => write!(f, "rad"),
            Grad => write!(f, "grad"),
            Turn => write!(f, "turn"),
            Percent => write!(f, "%"),
        }
    }
}

#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct PixelValue {
    pub metric: SizeMetric,
    pub number: FloatValue,
}

impl fmt::Debug for PixelValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.number, self.metric)
    }
}

// Manual Debug implementation, because the auto-generated one is nearly unreadable
impl fmt::Display for PixelValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.number, self.metric)
    }
}

impl fmt::Display for SizeMetric {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::SizeMetric::*;
        match self {
            Px => write!(f, "px"),
            Pt => write!(f, "pt"),
            Em => write!(f, "pt"),
            Percent => write!(f, "%"),
        }
    }
}

impl PixelValue {
    #[inline]
    pub const fn zero() -> Self {
        const ZERO_PX: PixelValue = PixelValue::const_px(0);
        ZERO_PX
    }

    /// Same as `PixelValue::px()`, but only accepts whole numbers,
    /// since using `f32` in const fn is not yet stabilized.
    #[inline]
    pub const fn const_px(value: isize) -> Self {
        Self::const_from_metric(SizeMetric::Px, value)
    }

    /// Same as `PixelValue::em()`, but only accepts whole numbers,
    /// since using `f32` in const fn is not yet stabilized.
    #[inline]
    pub const fn const_em(value: isize) -> Self {
        Self::const_from_metric(SizeMetric::Em, value)
    }

    /// Same as `PixelValue::pt()`, but only accepts whole numbers,
    /// since using `f32` in const fn is not yet stabilized.
    #[inline]
    pub const fn const_pt(value: isize) -> Self {
        Self::const_from_metric(SizeMetric::Pt, value)
    }

    /// Same as `PixelValue::pt()`, but only accepts whole numbers,
    /// since using `f32` in const fn is not yet stabilized.
    #[inline]
    pub const fn const_percent(value: isize) -> Self {
        Self::const_from_metric(SizeMetric::Percent, value)
    }

    #[inline]
    pub const fn const_from_metric(metric: SizeMetric, value: isize) -> Self {
        Self {
            metric: metric,
            number: FloatValue::const_new(value),
        }
    }

    #[inline]
    pub fn px(value: f32) -> Self {
        Self::from_metric(SizeMetric::Px, value)
    }

    #[inline]
    pub fn em(value: f32) -> Self {
        Self::from_metric(SizeMetric::Em, value)
    }

    #[inline]
    pub fn pt(value: f32) -> Self {
        Self::from_metric(SizeMetric::Pt, value)
    }

    #[inline]
    pub fn percent(value: f32) -> Self {
        Self::from_metric(SizeMetric::Percent, value)
    }

    #[inline]
    pub fn from_metric(metric: SizeMetric, value: f32) -> Self {
        Self {
            metric: metric,
            number: FloatValue::new(value),
        }
    }

    #[inline]
    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        if self.metric == other.metric {
            Self {
                metric: self.metric,
                number: self.number.interpolate(&other.number, t),
            }
        } else {
            // TODO: how to interpolate between different metrics
            // (interpolate between % and em? - currently impossible)
            let self_px_interp = self.to_pixels(0.0);
            let other_px_interp = other.to_pixels(0.0);
            Self::from_metric(
                SizeMetric::Px,
                self_px_interp + (other_px_interp - self_px_interp) * t,
            )
        }
    }

    /// Returns the value of the SizeMetric in pixels
    #[inline]
    pub fn to_pixels(&self, percent_resolve: f32) -> f32 {
        match self.metric {
            SizeMetric::Px => self.number.get(),
            SizeMetric::Pt => self.number.get() * PT_TO_PX,
            SizeMetric::Em => self.number.get() * EM_HEIGHT,
            SizeMetric::Percent => self.number.get() / 100.0 * percent_resolve,
        }
    }
}

/// Wrapper around FloatValue, represents a percentage instead
/// of just being a regular floating-point value, i.e `5` = `5%`
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct PercentageValue {
    number: FloatValue,
}

impl fmt::Display for PercentageValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}%", self.get())
    }
}

impl PercentageValue {
    /// Same as `PercentageValue::new()`, but only accepts whole numbers,
    /// since using `f32` in const fn is not yet stabilized.
    #[inline]
    pub const fn const_new(value: isize) -> Self {
        Self {
            number: FloatValue::const_new(value),
        }
    }

    #[inline]
    pub fn new(value: f32) -> Self {
        Self {
            number: value.into(),
        }
    }

    #[inline]
    pub fn get(&self) -> f32 {
        self.number.get()
    }

    #[inline]
    pub fn normalized(&self) -> f32 {
        self.get() / 100.0
    }

    #[inline]
    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        Self {
            number: self.number.interpolate(&other.number, t),
        }
    }
}

/// Wrapper around an f32 value that is internally casted to an isize,
/// in order to provide hash-ability (to avoid numerical instability).
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct FloatValue {
    pub number: isize,
}

impl fmt::Display for FloatValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get())
    }
}

impl ::core::fmt::Debug for FloatValue {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Default for FloatValue {
    fn default() -> Self {
        const DEFAULT_FLV: FloatValue = FloatValue::const_new(0);
        DEFAULT_FLV
    }
}

impl FloatValue {
    /// Same as `FloatValue::new()`, but only accepts whole numbers,
    /// since using `f32` in const fn is not yet stabilized.
    #[inline]
    pub const fn const_new(value: isize) -> Self {
        Self {
            number: value * FP_PRECISION_MULTIPLIER_CONST,
        }
    }

    #[inline]
    pub fn new(value: f32) -> Self {
        Self {
            number: (value * FP_PRECISION_MULTIPLIER) as isize,
        }
    }

    #[inline]
    pub fn get(&self) -> f32 {
        self.number as f32 / FP_PRECISION_MULTIPLIER
    }

    #[inline]
    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        let self_val_f32 = self.get();
        let other_val_f32 = other.get();
        let interpolated = self_val_f32 + ((other_val_f32 - self_val_f32) * t);
        Self::new(interpolated)
    }
}

impl From<f32> for FloatValue {
    #[inline]
    fn from(val: f32) -> Self {
        Self::new(val)
    }
}

/// Enum representing the metric associated with a number (px, pt, em, etc.)
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum SizeMetric {
    Px,
    Pt,
    Em,
    Percent,
}

impl Default for SizeMetric {
    fn default() -> Self {
        SizeMetric::Px
    }
}

/// Represents a `background-size` attribute
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, u8)]
pub enum StyleBackgroundSize {
    ExactSize([PixelValue; 2]),
    Contain,
    Cover,
}

impl Default for StyleBackgroundSize {
    fn default() -> Self {
        StyleBackgroundSize::Contain
    }
}

/// Represents a `background-position` attribute
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleBackgroundPosition {
    pub horizontal: BackgroundPositionHorizontal,
    pub vertical: BackgroundPositionVertical,
}

impl Default for StyleBackgroundPosition {
    fn default() -> Self {
        StyleBackgroundPosition {
            horizontal: BackgroundPositionHorizontal::Left,
            vertical: BackgroundPositionVertical::Top,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, u8)]
pub enum BackgroundPositionHorizontal {
    Left,
    Center,
    Right,
    Exact(PixelValue),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, u8)]
pub enum BackgroundPositionVertical {
    Top,
    Center,
    Bottom,
    Exact(PixelValue),
}

/// Represents a `background-repeat` attribute
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum StyleBackgroundRepeat {
    NoRepeat,
    Repeat,
    RepeatX,
    RepeatY,
}

impl Default for StyleBackgroundRepeat {
    fn default() -> Self {
        StyleBackgroundRepeat::Repeat
    }
}

/// Represents a `color` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleTextColor {
    pub inner: ColorU,
}

derive_debug_zero!(StyleTextColor);
derive_display_zero!(StyleTextColor);

impl StyleTextColor {
    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        Self {
            inner: self.inner.interpolate(&other.inner, t),
        }
    }
}

// -- TODO: Technically, border-radius can take two values for each corner!

/// Represents a `border-top-width` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleBorderTopLeftRadius {
    pub inner: PixelValue,
}
/// Represents a `border-left-width` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleBorderBottomLeftRadius {
    pub inner: PixelValue,
}
/// Represents a `border-right-width` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleBorderTopRightRadius {
    pub inner: PixelValue,
}
/// Represents a `border-bottom-width` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleBorderBottomRightRadius {
    pub inner: PixelValue,
}

impl_pixel_value!(StyleBorderTopLeftRadius);
impl_pixel_value!(StyleBorderBottomLeftRadius);
impl_pixel_value!(StyleBorderTopRightRadius);
impl_pixel_value!(StyleBorderBottomRightRadius);

/// Represents a `border-top-width` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LayoutBorderTopWidth {
    pub inner: PixelValue,
}
/// Represents a `border-left-width` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LayoutBorderLeftWidth {
    pub inner: PixelValue,
}
/// Represents a `border-right-width` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LayoutBorderRightWidth {
    pub inner: PixelValue,
}
/// Represents a `border-bottom-width` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LayoutBorderBottomWidth {
    pub inner: PixelValue,
}

impl_pixel_value!(LayoutBorderTopWidth);
impl_pixel_value!(LayoutBorderLeftWidth);
impl_pixel_value!(LayoutBorderRightWidth);
impl_pixel_value!(LayoutBorderBottomWidth);

/// Represents a `border-top-width` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleBorderTopStyle {
    pub inner: BorderStyle,
}
/// Represents a `border-left-width` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleBorderLeftStyle {
    pub inner: BorderStyle,
}
/// Represents a `border-right-width` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleBorderRightStyle {
    pub inner: BorderStyle,
}
/// Represents a `border-bottom-width` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleBorderBottomStyle {
    pub inner: BorderStyle,
}

derive_debug_zero!(StyleBorderTopStyle);
derive_debug_zero!(StyleBorderLeftStyle);
derive_debug_zero!(StyleBorderBottomStyle);
derive_debug_zero!(StyleBorderRightStyle);

derive_display_zero!(StyleBorderTopStyle);
derive_display_zero!(StyleBorderLeftStyle);
derive_display_zero!(StyleBorderBottomStyle);
derive_display_zero!(StyleBorderRightStyle);

/// Represents a `border-top-width` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleBorderTopColor {
    pub inner: ColorU,
}
/// Represents a `border-left-width` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleBorderLeftColor {
    pub inner: ColorU,
}
/// Represents a `border-right-width` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleBorderRightColor {
    pub inner: ColorU,
}
/// Represents a `border-bottom-width` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleBorderBottomColor {
    pub inner: ColorU,
}

impl StyleBorderTopColor {
    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        Self {
            inner: self.inner.interpolate(&other.inner, t),
        }
    }
}
impl StyleBorderLeftColor {
    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        Self {
            inner: self.inner.interpolate(&other.inner, t),
        }
    }
}
impl StyleBorderRightColor {
    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        Self {
            inner: self.inner.interpolate(&other.inner, t),
        }
    }
}
impl StyleBorderBottomColor {
    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        Self {
            inner: self.inner.interpolate(&other.inner, t),
        }
    }
}
derive_debug_zero!(StyleBorderTopColor);
derive_debug_zero!(StyleBorderLeftColor);
derive_debug_zero!(StyleBorderRightColor);
derive_debug_zero!(StyleBorderBottomColor);

derive_display_zero!(StyleBorderTopColor);
derive_display_zero!(StyleBorderLeftColor);
derive_display_zero!(StyleBorderRightColor);
derive_display_zero!(StyleBorderBottomColor);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StyleBorderSide {
    pub border_width: PixelValue,
    pub border_style: BorderStyle,
    pub border_color: ColorU,
}

// missing StyleBorderRadius & LayoutRect
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleBoxShadow {
    pub offset: [PixelValueNoPercent; 2],
    pub color: ColorU,
    pub blur_radius: PixelValueNoPercent,
    pub spread_radius: PixelValueNoPercent,
    pub clip_mode: BoxShadowClipMode,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, u8)]
pub enum StyleBackgroundContent {
    LinearGradient(LinearGradient),
    RadialGradient(RadialGradient),
    ConicGradient(ConicGradient),
    Image(String),
    Color(ColorU),
}

impl Default for StyleBackgroundContent {
    fn default() -> StyleBackgroundContent {
        StyleBackgroundContent::Color(ColorU::TRANSPARENT)
    }
}

impl<'a> From<String> for StyleBackgroundContent {
    fn from(id: String) -> Self {
        StyleBackgroundContent::Image(id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LinearGradient {
    pub direction: Direction,
    pub extend_mode: ExtendMode,
    pub stops: Vec<NormalizedLinearColorStop>,
}

impl Default for LinearGradient {
    fn default() -> Self {
        Self {
            direction: Direction::default(),
            extend_mode: ExtendMode::default(),
            stops: Vec::new().into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct ConicGradient {
    pub extend_mode: ExtendMode,               // default = clamp (no-repeat)
    pub center: StyleBackgroundPosition,       // default = center center
    pub angle: AngleValue,                     // default = 0deg
    pub stops: Vec<NormalizedRadialColorStop>, // default = []
}

impl Default for ConicGradient {
    fn default() -> Self {
        Self {
            extend_mode: ExtendMode::default(),
            center: StyleBackgroundPosition {
                horizontal: BackgroundPositionHorizontal::Center,
                vertical: BackgroundPositionVertical::Center,
            },
            angle: AngleValue::default(),
            stops: Vec::new().into(),
        }
    }
}

// normalized linear color stop
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct NormalizedLinearColorStop {
    pub offset: PercentageValue, // 0 to 100% // -- todo: theoretically this should be PixelValue
    pub color: ColorU,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct NormalizedRadialColorStop {
    pub angle: AngleValue, // 0 to 360 degrees
    pub color: ColorU,
}

impl LinearColorStop {
    pub fn get_normalized_linear_stops(
        stops: &[LinearColorStop],
    ) -> Vec<NormalizedLinearColorStop> {
        const MIN_STOP_DEGREE: f32 = 0.0;
        const MAX_STOP_DEGREE: f32 = 100.0;

        if stops.is_empty() {
            return Vec::new();
        }

        let self_stops = stops;

        let mut stops = self_stops
            .iter()
            .map(|s| NormalizedLinearColorStop {
                offset: s
                    .offset
                    .as_ref()
                    .copied()
                    .unwrap_or(PercentageValue::new(MIN_STOP_DEGREE)),
                color: s.color,
            })
            .collect::<Vec<_>>();

        let mut stops_to_distribute = 0;
        let mut last_stop = None;
        let stops_len = stops.len();

        for (stop_id, stop) in self_stops.iter().enumerate() {
            if let Some(s) = stop.offset {
                let current_stop_val = s.get();
                if stops_to_distribute != 0 {
                    let last_stop_val = stops[(stop_id - stops_to_distribute)].offset.get();
                    let value_to_add_per_stop = (current_stop_val.max(last_stop_val)
                        - last_stop_val)
                        / (stops_to_distribute - 1) as f32;
                    for (s_id, s) in stops[(stop_id - stops_to_distribute)..stop_id]
                        .iter_mut()
                        .enumerate()
                    {
                        s.offset = PercentageValue::new(
                            last_stop_val + (s_id as f32 * value_to_add_per_stop),
                        );
                    }
                }
                stops_to_distribute = 0;
                last_stop = Some(s);
            } else {
                stops_to_distribute += 1;
            }
        }

        if stops_to_distribute != 0 {
            let last_stop_val = last_stop
                .unwrap_or(PercentageValue::new(MIN_STOP_DEGREE))
                .get();
            let value_to_add_per_stop = (MAX_STOP_DEGREE.max(last_stop_val) - last_stop_val)
                / (stops_to_distribute - 1) as f32;
            for (s_id, s) in stops[(stops_len - stops_to_distribute)..]
                .iter_mut()
                .enumerate()
            {
                s.offset =
                    PercentageValue::new(last_stop_val + (s_id as f32 * value_to_add_per_stop));
            }
        }

        stops
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct RadialGradient {
    pub shape: Shape,
    pub size: RadialGradientSize,
    pub position: StyleBackgroundPosition,
    pub extend_mode: ExtendMode,
    pub stops: Vec<NormalizedLinearColorStop>,
}

impl Default for RadialGradient {
    fn default() -> Self {
        Self {
            shape: Shape::default(),
            size: RadialGradientSize::default(),
            position: StyleBackgroundPosition::default(),
            extend_mode: ExtendMode::default(),
            stops: Vec::new().into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(C)]
pub enum RadialGradientSize {
    // The gradient's ending shape meets the side of the box closest to its center
    // (for circles) or meets both the vertical and horizontal sides closest to the
    // center (for ellipses).
    ClosestSide,
    // The gradient's ending shape is sized so that it exactly meets the closest
    // corner of the box from its center
    ClosestCorner,
    // Similar to closest-side, except the ending shape is sized to meet the side
    // of the box farthest from its center (or vertical and horizontal sides)
    FarthestSide,
    // The default value, the gradient's ending shape is sized so that it exactly
    // meets the farthest corner of the box from its center
    #[default]
    FarthestCorner,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct DirectionCorners {
    pub from: DirectionCorner,
    pub to: DirectionCorner,
}

/// CSS direction (necessary for gradients). Can either be a fixed angle or
/// a direction ("to right" / "to left", etc.).
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, u8)]
pub enum Direction {
    Angle(AngleValue),
    FromTo(DirectionCorners),
}

impl Default for Direction {
    fn default() -> Self {
        Direction::FromTo(DirectionCorners {
            from: DirectionCorner::Top,
            to: DirectionCorner::Bottom,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum Shape {
    Ellipse,
    Circle,
}

impl Default for Shape {
    fn default() -> Self {
        Shape::Ellipse
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum StyleCursor {
    /// `alias`
    Alias,
    /// `all-scroll`
    AllScroll,
    /// `cell`
    Cell,
    /// `col-resize`
    ColResize,
    /// `context-menu`
    ContextMenu,
    /// `copy`
    Copy,
    /// `crosshair`
    Crosshair,
    /// `default` - note: called "arrow" in winit
    Default,
    /// `e-resize`
    EResize,
    /// `ew-resize`
    EwResize,
    /// `grab`
    Grab,
    /// `grabbing`
    Grabbing,
    /// `help`
    Help,
    /// `move`
    Move,
    /// `n-resize`
    NResize,
    /// `ns-resize`
    NsResize,
    /// `nesw-resize`
    NeswResize,
    /// `nwse-resize`
    NwseResize,
    /// `pointer` - note: called "hand" in winit
    Pointer,
    /// `progress`
    Progress,
    /// `row-resize`
    RowResize,
    /// `s-resize`
    SResize,
    /// `se-resize`
    SeResize,
    /// `text`
    Text,
    /// `unset`
    Unset,
    /// `vertical-text`
    VerticalText,
    /// `w-resize`
    WResize,
    /// `wait`
    Wait,
    /// `zoom-in`
    ZoomIn,
    /// `zoom-out`
    ZoomOut,
}

impl Default for StyleCursor {
    fn default() -> StyleCursor {
        StyleCursor::Default
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum DirectionCorner {
    Right,
    Left,
    Top,
    Bottom,
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

impl fmt::Display for DirectionCorner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DirectionCorner::Right => "right",
                DirectionCorner::Left => "left",
                DirectionCorner::Top => "top",
                DirectionCorner::Bottom => "bottom",
                DirectionCorner::TopRight => "top right",
                DirectionCorner::TopLeft => "top left",
                DirectionCorner::BottomRight => "bottom right",
                DirectionCorner::BottomLeft => "bottom left",
            }
        )
    }
}

impl DirectionCorner {
    pub const fn opposite(&self) -> Self {
        use self::DirectionCorner::*;
        match *self {
            Right => Left,
            Left => Right,
            Top => Bottom,
            Bottom => Top,
            TopRight => BottomLeft,
            BottomLeft => TopRight,
            TopLeft => BottomRight,
            BottomRight => TopLeft,
        }
    }

    pub const fn combine(&self, other: &Self) -> Option<Self> {
        use self::DirectionCorner::*;
        match (*self, *other) {
            (Right, Top) | (Top, Right) => Some(TopRight),
            (Left, Top) | (Top, Left) => Some(TopLeft),
            (Right, Bottom) | (Bottom, Right) => Some(BottomRight),
            (Left, Bottom) | (Bottom, Left) => Some(BottomLeft),
            _ => None,
        }
    }

    pub const fn to_point(&self, rect: &LayoutRect) -> LayoutPoint {
        use self::DirectionCorner::*;
        match *self {
            Right => LayoutPoint {
                x: rect.size.width,
                y: rect.size.height / 2,
            },
            Left => LayoutPoint {
                x: 0,
                y: rect.size.height / 2,
            },
            Top => LayoutPoint {
                x: rect.size.width / 2,
                y: 0,
            },
            Bottom => LayoutPoint {
                x: rect.size.width / 2,
                y: rect.size.height,
            },
            TopRight => LayoutPoint {
                x: rect.size.width,
                y: 0,
            },
            TopLeft => LayoutPoint { x: 0, y: 0 },
            BottomRight => LayoutPoint {
                x: rect.size.width,
                y: rect.size.height,
            },
            BottomLeft => LayoutPoint {
                x: 0,
                y: rect.size.height,
            },
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RadialColorStop {
    // this is set to None if there was no offset that could be parsed
    pub offset: Option<AngleValue>,
    pub color: ColorU,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LinearColorStop {
    // this is set to None if there was no offset that could be parsed
    pub offset: Option<PercentageValue>,
    pub color: ColorU,
}

/// Represents a `width` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LayoutWidth {
    pub inner: PixelValue,
}
/// Represents a `min-width` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LayoutMinWidth {
    pub inner: PixelValue,
}
/// Represents a `max-width` attribute
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LayoutMaxWidth {
    pub inner: PixelValue,
}
/// Represents a `height` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LayoutHeight {
    pub inner: PixelValue,
}
/// Represents a `min-height` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LayoutMinHeight {
    pub inner: PixelValue,
}
/// Represents a `max-height` attribute
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LayoutMaxHeight {
    pub inner: PixelValue,
}

impl Default for LayoutMaxHeight {
    fn default() -> Self {
        Self {
            inner: PixelValue::px(core::f32::MAX),
        }
    }
}
impl Default for LayoutMaxWidth {
    fn default() -> Self {
        Self {
            inner: PixelValue::px(core::f32::MAX),
        }
    }
}

impl_pixel_value!(LayoutWidth);
impl_pixel_value!(LayoutHeight);
impl_pixel_value!(LayoutMinHeight);
impl_pixel_value!(LayoutMinWidth);
impl_pixel_value!(LayoutMaxWidth);
impl_pixel_value!(LayoutMaxHeight);

/// Represents a `top` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LayoutTop {
    pub inner: PixelValue,
}
/// Represents a `left` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LayoutLeft {
    pub inner: PixelValue,
}
/// Represents a `right` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LayoutRight {
    pub inner: PixelValue,
}
/// Represents a `bottom` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LayoutBottom {
    pub inner: PixelValue,
}

impl_pixel_value!(LayoutTop);
impl_pixel_value!(LayoutBottom);
impl_pixel_value!(LayoutRight);
impl_pixel_value!(LayoutLeft);

/// Represents a `padding-top` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LayoutPaddingTop {
    pub inner: PixelValue,
}
/// Represents a `padding-left` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LayoutPaddingLeft {
    pub inner: PixelValue,
}
/// Represents a `padding-right` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LayoutPaddingRight {
    pub inner: PixelValue,
}
/// Represents a `padding-bottom` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LayoutPaddingBottom {
    pub inner: PixelValue,
}

impl_pixel_value!(LayoutPaddingTop);
impl_pixel_value!(LayoutPaddingBottom);
impl_pixel_value!(LayoutPaddingRight);
impl_pixel_value!(LayoutPaddingLeft);

/// Represents a `padding-top` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LayoutMarginTop {
    pub inner: PixelValue,
}
/// Represents a `padding-left` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LayoutMarginLeft {
    pub inner: PixelValue,
}
/// Represents a `padding-right` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LayoutMarginRight {
    pub inner: PixelValue,
}
/// Represents a `padding-bottom` attribute
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LayoutMarginBottom {
    pub inner: PixelValue,
}

impl_pixel_value!(LayoutMarginTop);
impl_pixel_value!(LayoutMarginBottom);
impl_pixel_value!(LayoutMarginRight);
impl_pixel_value!(LayoutMarginLeft);

/// Represents a `flex-grow` attribute
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LayoutFlexGrow {
    pub inner: FloatValue,
}

impl Default for LayoutFlexGrow {
    fn default() -> Self {
        LayoutFlexGrow {
            inner: FloatValue::const_new(0),
        }
    }
}

/// Represents a `flex-shrink` attribute
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct LayoutFlexShrink {
    pub inner: FloatValue,
}

impl Default for LayoutFlexShrink {
    fn default() -> Self {
        LayoutFlexShrink {
            inner: FloatValue::const_new(0),
        }
    }
}

impl_float_value!(LayoutFlexGrow);
impl_float_value!(LayoutFlexShrink);

/// Represents a `flex-direction` attribute - default: `Column`
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum LayoutFlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

impl Default for LayoutFlexDirection {
    fn default() -> Self {
        LayoutFlexDirection::Column
    }
}

impl LayoutFlexDirection {
    pub fn get_axis(&self) -> LayoutAxis {
        use self::{LayoutAxis::*, LayoutFlexDirection::*};
        match self {
            Row | RowReverse => Horizontal,
            Column | ColumnReverse => Vertical,
        }
    }

    /// Returns true, if this direction is a `column-reverse` or `row-reverse` direction
    pub fn is_reverse(&self) -> bool {
        *self == LayoutFlexDirection::RowReverse || *self == LayoutFlexDirection::ColumnReverse
    }
}

/// Represents a `flex-direction` attribute - default: `Column`
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum LayoutBoxSizing {
    ContentBox,
    BorderBox,
}

impl Default for LayoutBoxSizing {
    fn default() -> Self {
        LayoutBoxSizing::ContentBox
    }
}

/// Represents a `line-height` attribute
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleLineHeight {
    pub inner: PercentageValue,
}

impl_percentage_value!(StyleLineHeight);

impl Default for StyleLineHeight {
    fn default() -> Self {
        Self {
            inner: PercentageValue::const_new(100),
        }
    }
}

/// Represents a `tab-width` attribute
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleTabWidth {
    pub inner: PercentageValue,
}

impl_percentage_value!(StyleTabWidth);

impl Default for StyleTabWidth {
    fn default() -> Self {
        Self {
            inner: PercentageValue::const_new(100),
        }
    }
}

/// Represents a `letter-spacing` attribute
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleLetterSpacing {
    pub inner: PixelValue,
}

impl Default for StyleLetterSpacing {
    fn default() -> Self {
        Self {
            inner: PixelValue::const_px(0),
        }
    }
}

impl_pixel_value!(StyleLetterSpacing);

/// Represents a `word-spacing` attribute
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleWordSpacing {
    pub inner: PixelValue,
}

impl_pixel_value!(StyleWordSpacing);

impl Default for StyleWordSpacing {
    fn default() -> Self {
        Self {
            inner: PixelValue::const_px(0),
        }
    }
}

/// Same as the `LayoutFlexDirection`, but without the `-reverse` properties, used in the layout solver,
/// makes decisions based on horizontal / vertical direction easier to write.
/// Use `LayoutFlexDirection::get_axis()` to get the axis for a given `LayoutFlexDirection`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum LayoutAxis {
    Horizontal,
    Vertical,
}

/// Represents a `display` attribute
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum LayoutDisplay {
    None,
    Flex,
    Block,
    InlineBlock,
}

impl Default for LayoutDisplay {
    fn default() -> Self {
        LayoutDisplay::Flex
    }
}

/// Represents a `float` attribute
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum LayoutFloat {
    Left,
    Right,
}

impl Default for LayoutFloat {
    fn default() -> Self {
        LayoutFloat::Left
    }
}

/// Represents a `position` attribute - default: `Static`
///
/// NOTE: No inline positioning is supported.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum LayoutPosition {
    Static,
    Relative,
    Absolute,
    Fixed,
}

impl LayoutPosition {
    pub fn is_positioned(&self) -> bool {
        *self != LayoutPosition::Static
    }
}

impl Default for LayoutPosition {
    fn default() -> Self {
        LayoutPosition::Static
    }
}

/// Represents a `flex-wrap` attribute - default: `Wrap`
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum LayoutFlexWrap {
    Wrap,
    NoWrap,
}

impl Default for LayoutFlexWrap {
    fn default() -> Self {
        LayoutFlexWrap::Wrap
    }
}

/// Represents a `justify-content` attribute
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum LayoutJustifyContent {
    /// Default value. Items are positioned at the beginning of the container
    Start,
    /// Items are positioned at the end of the container
    End,
    /// Items are positioned at the center of the container
    Center,
    /// Items are positioned with space between the lines
    SpaceBetween,
    /// Items are positioned with space before, between, and after the lines
    SpaceAround,
    /// Items are distributed so that the spacing between any two adjacent alignment subjects,
    /// before the first alignment subject, and after the last alignment subject is the same
    SpaceEvenly,
}

impl Default for LayoutJustifyContent {
    fn default() -> Self {
        LayoutJustifyContent::Start
    }
}

/// Represents a `align-items` attribute
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum LayoutAlignItems {
    /// Items are stretched to fit the container
    Stretch,
    /// Items are positioned at the center of the container
    Center,
    /// Items are positioned at the beginning of the container
    FlexStart,
    /// Items are positioned at the end of the container
    FlexEnd,
}

impl Default for LayoutAlignItems {
    fn default() -> Self {
        LayoutAlignItems::FlexStart
    }
}

/// Represents a `align-content` attribute
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum LayoutAlignContent {
    /// Default value. Lines stretch to take up the remaining space
    Stretch,
    /// Lines are packed toward the center of the flex container
    Center,
    /// Lines are packed toward the start of the flex container
    Start,
    /// Lines are packed toward the end of the flex container
    End,
    /// Lines are evenly distributed in the flex container
    SpaceBetween,
    /// Lines are evenly distributed in the flex container, with half-size spaces on either end
    SpaceAround,
}

impl Default for LayoutAlignContent {
    fn default() -> Self {
        LayoutAlignContent::Stretch
    }
}

/// Represents a `overflow-x` or `overflow-y` property, see
/// [`TextOverflowBehaviour`](./struct.TextOverflowBehaviour.html) - default: `Auto`
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum LayoutOverflow {
    /// Always shows a scroll bar, overflows on scroll
    Scroll,
    /// Does not show a scroll bar by default, only when text is overflowing
    Auto,
    /// Never shows a scroll bar, simply clips text
    Hidden,
    /// Doesn't show a scroll bar, simply overflows the text
    Visible,
}

impl Default for LayoutOverflow {
    fn default() -> Self {
        LayoutOverflow::Auto
    }
}

impl LayoutOverflow {
    /// Returns whether this overflow value needs to display the scrollbars.
    ///
    /// - `overflow:scroll` always shows the scrollbar
    /// - `overflow:auto` only shows the scrollbar when the content is currently overflowing
    /// - `overflow:hidden` and `overflow:visible` do not show any scrollbars
    pub fn needs_scrollbar(&self, currently_overflowing: bool) -> bool {
        use self::LayoutOverflow::*;
        match self {
            Scroll => true,
            Auto => currently_overflowing,
            Hidden | Visible => false,
        }
    }

    /// Returns whether this is an `overflow:visible` node
    /// (the only overflow type that doesn't clip its children)
    pub fn is_overflow_visible(&self) -> bool {
        *self == LayoutOverflow::Visible
    }

    pub fn is_overflow_hidden(&self) -> bool {
        *self == LayoutOverflow::Hidden
    }
}

/// Horizontal text alignment enum (left, center, right) - default: `Center`
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum StyleTextAlign {
    Left,
    Center,
    Right,
}

impl Default for StyleTextAlign {
    fn default() -> Self {
        StyleTextAlign::Left
    }
}

/// Vertical text alignment enum (top, center, bottom) - default: `Center`
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum StyleVerticalAlign {
    Top,
    Center,
    Bottom,
}

impl Default for StyleVerticalAlign {
    fn default() -> Self {
        StyleVerticalAlign::Top
    }
}

/// Represents an `opacity` attribute
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleOpacity {
    pub inner: PercentageValue,
}

impl Default for StyleOpacity {
    fn default() -> Self {
        StyleOpacity {
            inner: PercentageValue::const_new(0),
        }
    }
}

impl_percentage_value!(StyleOpacity);

/// Represents a `perspective-origin` attribute
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StylePerspectiveOrigin {
    pub x: PixelValue,
    pub y: PixelValue,
}

impl StylePerspectiveOrigin {
    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        Self {
            x: self.x.interpolate(&other.x, t),
            y: self.y.interpolate(&other.y, t),
        }
    }
}

impl Default for StylePerspectiveOrigin {
    fn default() -> Self {
        StylePerspectiveOrigin {
            x: PixelValue::const_px(0),
            y: PixelValue::const_px(0),
        }
    }
}

/// Represents a `transform-origin` attribute
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleTransformOrigin {
    pub x: PixelValue,
    pub y: PixelValue,
}

impl StyleTransformOrigin {
    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        Self {
            x: self.x.interpolate(&other.x, t),
            y: self.y.interpolate(&other.y, t),
        }
    }
}

impl Default for StyleTransformOrigin {
    fn default() -> Self {
        StyleTransformOrigin {
            x: PixelValue::const_percent(50),
            y: PixelValue::const_percent(50),
        }
    }
}

/// Represents a `backface-visibility` attribute
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum StyleBackfaceVisibility {
    Hidden,
    Visible,
}

impl Default for StyleBackfaceVisibility {
    fn default() -> Self {
        StyleBackfaceVisibility::Visible
    }
}

/// Represents an `opacity` attribute
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, u8)]
pub enum StyleTransform {
    Matrix(StyleTransformMatrix2D),
    Matrix3D(StyleTransformMatrix3D),
    Translate(StyleTransformTranslate2D),
    Translate3D(StyleTransformTranslate3D),
    TranslateX(PixelValue),
    TranslateY(PixelValue),
    TranslateZ(PixelValue),
    Rotate(AngleValue),
    Rotate3D(StyleTransformRotate3D),
    RotateX(AngleValue),
    RotateY(AngleValue),
    RotateZ(AngleValue),
    Scale(StyleTransformScale2D),
    Scale3D(StyleTransformScale3D),
    ScaleX(PercentageValue),
    ScaleY(PercentageValue),
    ScaleZ(PercentageValue),
    Skew(StyleTransformSkew2D),
    SkewX(PercentageValue),
    SkewY(PercentageValue),
    Perspective(PixelValue),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleTransformMatrix2D {
    pub a: PixelValue,
    pub b: PixelValue,
    pub c: PixelValue,
    pub d: PixelValue,
    pub tx: PixelValue,
    pub ty: PixelValue,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleTransformMatrix3D {
    pub m11: PixelValue,
    pub m12: PixelValue,
    pub m13: PixelValue,
    pub m14: PixelValue,
    pub m21: PixelValue,
    pub m22: PixelValue,
    pub m23: PixelValue,
    pub m24: PixelValue,
    pub m31: PixelValue,
    pub m32: PixelValue,
    pub m33: PixelValue,
    pub m34: PixelValue,
    pub m41: PixelValue,
    pub m42: PixelValue,
    pub m43: PixelValue,
    pub m44: PixelValue,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleTransformTranslate2D {
    pub x: PixelValue,
    pub y: PixelValue,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleTransformTranslate3D {
    pub x: PixelValue,
    pub y: PixelValue,
    pub z: PixelValue,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleTransformRotate3D {
    pub x: PercentageValue,
    pub y: PercentageValue,
    pub z: PercentageValue,
    pub angle: AngleValue,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleTransformScale2D {
    pub x: PercentageValue,
    pub y: PercentageValue,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleTransformScale3D {
    pub x: PercentageValue,
    pub y: PercentageValue,
    pub z: PercentageValue,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleTransformSkew2D {
    pub x: PercentageValue,
    pub y: PercentageValue,
}

pub type StyleBackgroundContentVecValue = CssPropertyValue<Vec<StyleBackgroundContent>>;
pub type StyleBackgroundPositionVecValue = CssPropertyValue<Vec<StyleBackgroundPosition>>;
pub type StyleBackgroundSizeVecValue = CssPropertyValue<Vec<StyleBackgroundSize>>;
pub type StyleBackgroundRepeatVecValue = CssPropertyValue<Vec<StyleBackgroundRepeat>>;
pub type StyleFontSizeValue = CssPropertyValue<StyleFontSize>;
pub type StyleFontFamilyVecValue = CssPropertyValue<Vec<StyleFontFamily>>;
pub type StyleTextColorValue = CssPropertyValue<StyleTextColor>;
pub type StyleTextAlignValue = CssPropertyValue<StyleTextAlign>;
pub type StyleLineHeightValue = CssPropertyValue<StyleLineHeight>;
pub type StyleLetterSpacingValue = CssPropertyValue<StyleLetterSpacing>;
pub type StyleWordSpacingValue = CssPropertyValue<StyleWordSpacing>;
pub type StyleTabWidthValue = CssPropertyValue<StyleTabWidth>;
pub type StyleCursorValue = CssPropertyValue<StyleCursor>;
pub type StyleBoxShadowValue = CssPropertyValue<StyleBoxShadow>;
pub type StyleBorderTopColorValue = CssPropertyValue<StyleBorderTopColor>;
pub type StyleBorderLeftColorValue = CssPropertyValue<StyleBorderLeftColor>;
pub type StyleBorderRightColorValue = CssPropertyValue<StyleBorderRightColor>;
pub type StyleBorderBottomColorValue = CssPropertyValue<StyleBorderBottomColor>;
pub type StyleBorderTopStyleValue = CssPropertyValue<StyleBorderTopStyle>;
pub type StyleBorderLeftStyleValue = CssPropertyValue<StyleBorderLeftStyle>;
pub type StyleBorderRightStyleValue = CssPropertyValue<StyleBorderRightStyle>;
pub type StyleBorderBottomStyleValue = CssPropertyValue<StyleBorderBottomStyle>;
pub type StyleBorderTopLeftRadiusValue = CssPropertyValue<StyleBorderTopLeftRadius>;
pub type StyleBorderTopRightRadiusValue = CssPropertyValue<StyleBorderTopRightRadius>;
pub type StyleBorderBottomLeftRadiusValue = CssPropertyValue<StyleBorderBottomLeftRadius>;
pub type StyleBorderBottomRightRadiusValue = CssPropertyValue<StyleBorderBottomRightRadius>;
pub type StyleOpacityValue = CssPropertyValue<StyleOpacity>;
pub type StyleTransformVecValue = CssPropertyValue<Vec<StyleTransform>>;
pub type StyleTransformOriginValue = CssPropertyValue<StyleTransformOrigin>;
pub type StylePerspectiveOriginValue = CssPropertyValue<StylePerspectiveOrigin>;
pub type StyleBackfaceVisibilityValue = CssPropertyValue<StyleBackfaceVisibility>;
pub type StyleMixBlendModeValue = CssPropertyValue<StyleMixBlendMode>;
pub type StyleFilterVecValue = CssPropertyValue<Vec<StyleFilter>>;
pub type ScrollbarStyleValue = CssPropertyValue<ScrollbarStyle>;
pub type LayoutDisplayValue = CssPropertyValue<LayoutDisplay>;
pub type LayoutFloatValue = CssPropertyValue<LayoutFloat>;
pub type LayoutBoxSizingValue = CssPropertyValue<LayoutBoxSizing>;
pub type LayoutWidthValue = CssPropertyValue<LayoutWidth>;
pub type LayoutHeightValue = CssPropertyValue<LayoutHeight>;
pub type LayoutMinWidthValue = CssPropertyValue<LayoutMinWidth>;
pub type LayoutMinHeightValue = CssPropertyValue<LayoutMinHeight>;
pub type LayoutMaxWidthValue = CssPropertyValue<LayoutMaxWidth>;
pub type LayoutMaxHeightValue = CssPropertyValue<LayoutMaxHeight>;
pub type LayoutPositionValue = CssPropertyValue<LayoutPosition>;
pub type LayoutTopValue = CssPropertyValue<LayoutTop>;
pub type LayoutBottomValue = CssPropertyValue<LayoutBottom>;
pub type LayoutRightValue = CssPropertyValue<LayoutRight>;
pub type LayoutLeftValue = CssPropertyValue<LayoutLeft>;
pub type LayoutPaddingTopValue = CssPropertyValue<LayoutPaddingTop>;
pub type LayoutPaddingBottomValue = CssPropertyValue<LayoutPaddingBottom>;
pub type LayoutPaddingLeftValue = CssPropertyValue<LayoutPaddingLeft>;
pub type LayoutPaddingRightValue = CssPropertyValue<LayoutPaddingRight>;
pub type LayoutMarginTopValue = CssPropertyValue<LayoutMarginTop>;
pub type LayoutMarginBottomValue = CssPropertyValue<LayoutMarginBottom>;
pub type LayoutMarginLeftValue = CssPropertyValue<LayoutMarginLeft>;
pub type LayoutMarginRightValue = CssPropertyValue<LayoutMarginRight>;
pub type LayoutBorderTopWidthValue = CssPropertyValue<LayoutBorderTopWidth>;
pub type LayoutBorderLeftWidthValue = CssPropertyValue<LayoutBorderLeftWidth>;
pub type LayoutBorderRightWidthValue = CssPropertyValue<LayoutBorderRightWidth>;
pub type LayoutBorderBottomWidthValue = CssPropertyValue<LayoutBorderBottomWidth>;
pub type LayoutOverflowValue = CssPropertyValue<LayoutOverflow>;
pub type LayoutFlexDirectionValue = CssPropertyValue<LayoutFlexDirection>;
pub type LayoutFlexWrapValue = CssPropertyValue<LayoutFlexWrap>;
pub type LayoutFlexGrowValue = CssPropertyValue<LayoutFlexGrow>;
pub type LayoutFlexShrinkValue = CssPropertyValue<LayoutFlexShrink>;
pub type LayoutJustifyContentValue = CssPropertyValue<LayoutJustifyContent>;
pub type LayoutAlignItemsValue = CssPropertyValue<LayoutAlignItems>;
pub type LayoutAlignContentValue = CssPropertyValue<LayoutAlignContent>;

/// Holds info necessary for layouting / styling scrollbars (-webkit-scrollbar)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct ScrollbarInfo {
    /// Total width (or height for vertical scrollbars) of the scrollbar in pixels
    pub width: LayoutWidth,
    /// Padding of the scrollbar tracker, in pixels. The inner bar is `width - padding` pixels wide.
    pub padding_left: LayoutPaddingLeft,
    /// Padding of the scrollbar (right)
    pub padding_right: LayoutPaddingRight,
    /// Style of the scrollbar background
    /// (`-webkit-scrollbar` / `-webkit-scrollbar-track` / `-webkit-scrollbar-track-piece` combined)
    pub track: StyleBackgroundContent,
    /// Style of the scrollbar thumbs (the "up" / "down" arrows), (`-webkit-scrollbar-thumb`)
    pub thumb: StyleBackgroundContent,
    /// Styles the directional buttons on the scrollbar (`-webkit-scrollbar-button`)
    pub button: StyleBackgroundContent,
    /// If two scrollbars are present, addresses the (usually) bottom corner
    /// of the scrollable element, where two scrollbars might meet (`-webkit-scrollbar-corner`)
    pub corner: StyleBackgroundContent,
    /// Addresses the draggable resizing handle that appears above the
    /// `corner` at the bottom corner of some elements (`-webkit-resizer`)
    pub resizer: StyleBackgroundContent,
}

impl Default for ScrollbarInfo {
    fn default() -> Self {
        ScrollbarInfo {
            width: LayoutWidth::px(17.0),
            padding_left: LayoutPaddingLeft::px(2.0),
            padding_right: LayoutPaddingRight::px(2.0),
            track: StyleBackgroundContent::Color(ColorU {
                r: 241,
                g: 241,
                b: 241,
                a: 255,
            }),
            thumb: StyleBackgroundContent::Color(ColorU {
                r: 193,
                g: 193,
                b: 193,
                a: 255,
            }),
            button: StyleBackgroundContent::Color(ColorU {
                r: 163,
                g: 163,
                b: 163,
                a: 255,
            }),
            corner: StyleBackgroundContent::default(),
            resizer: StyleBackgroundContent::default(),
        }
    }
}

/// Scrollbar style
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct ScrollbarStyle {
    /// Vertical scrollbar style, if any
    pub horizontal: ScrollbarInfo,
    /// Horizontal scrollbar style, if any
    pub vertical: ScrollbarInfo,
}

/// Represents a `font-size` attribute
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleFontSize {
    pub inner: PixelValue,
}

impl Default for StyleFontSize {
    fn default() -> Self {
        Self {
            inner: PixelValue::const_em(1),
        }
    }
}

impl_pixel_value!(StyleFontSize);

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

impl Default for FontMetrics {
    fn default() -> Self {
        FontMetrics::zero()
    }
}

impl FontMetrics {
    /// Only for testing, zero-sized font, will always return 0 for every metric (`units_per_em = 1000`)
    pub const fn zero() -> Self {
        FontMetrics {
            units_per_em: 1000,
            font_flags: 0,
            x_min: 0,
            y_min: 0,
            x_max: 0,
            y_max: 0,
            ascender: 0,
            descender: 0,
            line_gap: 0,
            advance_width_max: 0,
            min_left_side_bearing: 0,
            min_right_side_bearing: 0,
            x_max_extent: 0,
            caret_slope_rise: 0,
            caret_slope_run: 0,
            caret_offset: 0,
            num_h_metrics: 0,
            x_avg_char_width: 0,
            us_weight_class: 0,
            us_width_class: 0,
            fs_type: 0,
            y_subscript_x_size: 0,
            y_subscript_y_size: 0,
            y_subscript_x_offset: 0,
            y_subscript_y_offset: 0,
            y_superscript_x_size: 0,
            y_superscript_y_size: 0,
            y_superscript_x_offset: 0,
            y_superscript_y_offset: 0,
            y_strikeout_size: 0,
            y_strikeout_position: 0,
            s_family_class: 0,
            panose: [0; 10],
            ul_unicode_range1: 0,
            ul_unicode_range2: 0,
            ul_unicode_range3: 0,
            ul_unicode_range4: 0,
            ach_vend_id: 0,
            fs_selection: 0,
            us_first_char_index: 0,
            us_last_char_index: 0,
            s_typo_ascender: None,
            s_typo_descender: None,
            s_typo_line_gap: None,
            us_win_ascent: None,
            us_win_descent: None,
            ul_code_page_range1: None,
            ul_code_page_range2: None,
            sx_height: None,
            s_cap_height: None,
            us_default_char: None,
            us_break_char: None,
            us_max_context: None,
            us_lower_optical_point_size: None,
            us_upper_optical_point_size: None,
        }
    }

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
            self.s_typo_ascender.into()
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
            self.s_typo_descender.into()
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
            self.s_typo_line_gap.into()
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

#[derive(Debug, Clone)]
pub struct FontRef {
    /// shared pointer to an opaque implementation of the parsed font
    pub data: Arc<FontData>,
}

impl FontRef {
    #[inline]
    pub fn get_data<'a>(&'a self) -> &'a FontData {
        &*self.data
    }
}

impl FontRef {
    pub fn new(data: FontData) -> Self {
        Self {
            data: Arc::new(data),
        }
    }
    pub fn get_bytes(&self) -> Vec<u8> {
        self.get_data().bytes.clone()
    }
}

impl PartialEq for FontRef {
    fn eq(&self, rhs: &Self) -> bool {
        Arc::ptr_eq(&self.data, &rhs.data)
    }
}

impl PartialOrd for FontRef {
    fn partial_cmp(&self, other: &Self) -> Option<::core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FontRef {
    fn cmp(&self, other: &Self) -> Ordering {
        let a = Arc::as_ptr(&self.data) as usize;
        let b = Arc::as_ptr(&other.data) as usize;
        a.cmp(&b)
    }
}

impl Eq for FontRef {}

impl Hash for FontRef {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        let a = Arc::as_ptr(&self.data) as usize;
        a.hash(state)
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

/// Represents a `font-family` attribute
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StyleFontFamily {
    /// Native font, such as "Webly Sleeky UI", "monospace", etc.
    System(String),
    /// Font loaded from a file
    File(String),
    /// Reference-counted, already-decoded font,
    /// so that specific DOM nodes are required to use this font
    Ref(FontRef),
}

impl StyleFontFamily {
    pub(crate) fn as_string(&self) -> String {
        match &self {
            StyleFontFamily::System(s) => s.clone(),
            StyleFontFamily::File(s) => s.clone(),
            StyleFontFamily::Ref(s) => format!("{:p}", &s.data),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum StyleMixBlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl Default for StyleMixBlendMode {
    fn default() -> StyleMixBlendMode {
        StyleMixBlendMode::Normal
    }
}

impl fmt::Display for StyleMixBlendMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::StyleMixBlendMode::*;
        write!(
            f,
            "{}",
            match self {
                Normal => "normal",
                Multiply => "multiply",
                Screen => "screen",
                Overlay => "overlay",
                Darken => "darken",
                Lighten => "lighten",
                ColorDodge => "color-dodge",
                ColorBurn => "color-burn",
                HardLight => "hard-light",
                SoftLight => "soft-light",
                Difference => "difference",
                Exclusion => "exclusion",
                Hue => "hue",
                Saturation => "saturation",
                Color => "color",
                Luminosity => "luminosity",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, u8)]
pub enum StyleFilter {
    Blend(StyleMixBlendMode),
    Flood(ColorU),
    Blur(StyleBlur),
    Opacity(PercentageValue),
    ColorMatrix(StyleColorMatrix),
    DropShadow(StyleBoxShadow),
    ComponentTransfer,
    Offset(StyleFilterOffset),
    Composite(StyleCompositeFilter),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleBlur {
    pub width: PixelValue,
    pub height: PixelValue,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleColorMatrix {
    pub matrix: [FloatValue; 20],
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct StyleFilterOffset {
    pub x: PixelValue,
    pub y: PixelValue,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, u8)]
pub enum StyleCompositeFilter {
    Over,
    In,
    Atop,
    Out,
    Xor,
    Lighter,
    Arithmetic([FloatValue; 4]),
}
