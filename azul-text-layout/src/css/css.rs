//! Types and methods used to describe the style of an application
use crate::css::css_properties::CssProperty;
use core::fmt;

/// A `DynamicCssProperty` is a type of css property that can be changed on possibly
/// every frame by the Rust code - for example to implement an `On::Hover` behaviour.
///
/// The syntax for such a property looks like this:
///
/// ```no_run,ignore
/// #my_div {
///    padding: var(--my_dynamic_property_id, 400px);
/// }
/// ```
///
/// Azul will register a dynamic property with the key "my_dynamic_property_id"
/// and the default value of 400px. If the property gets overridden during one frame,
/// the overridden property takes precedence.
///
/// At runtime the style is immutable (which is a performance optimization - if we
/// can assume that the property never changes at runtime), we can do some optimizations on it.
/// Dynamic style properties can also be used for animations and conditional styles
/// (i.e. `hover`, `focus`, etc.), thereby leading to cleaner code, since all of these
/// special cases now use one single API.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(C)]
pub struct DynamicCssProperty {
    /// The stringified ID of this property, i.e. the `"my_id"` in `width: var(--my_id, 500px)`.
    pub dynamic_id: String,
    /// Default values for this properties - one single value can control multiple properties!
    pub default_value: CssProperty,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(C, u8)] // necessary for ABI stability
pub enum CssPropertyValue<T> {
    Exact(T),
}

pub trait PrintAsCssValue {
    fn print_as_css_value(&self) -> String;
}

impl<T: fmt::Display> fmt::Display for CssPropertyValue<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::CssPropertyValue::*;
        match self {
            Exact(e) => write!(f, "{}", e),
        }
    }
}

impl<T> From<T> for CssPropertyValue<T> {
    fn from(c: T) -> Self {
        CssPropertyValue::Exact(c)
    }
}

impl<T> CssPropertyValue<T> {}

impl<T: Default> Default for CssPropertyValue<T> {
    #[inline]
    fn default() -> Self {
        CssPropertyValue::Exact(T::default())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(C)]
pub struct CssNthChildPattern {
    pub repeat: u32,
    pub offset: u32,
}
