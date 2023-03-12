//! Types and methods used to describe the style of an application

use core::fmt;

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
