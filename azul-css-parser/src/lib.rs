//! Provides a reference implementation of a style parser for Azul, capable of parsing CSS
//! stylesheets into their respective `Css` counterparts.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/maps4print/azul/master/assets/images/azul_logo_full_min.svg.png",
    html_favicon_url = "https://raw.githubusercontent.com/maps4print/azul/master/assets/images/favicon.ico"
)]

#[macro_use]
mod macros;

mod css;
mod css_parser;

pub use crate::css::*;
pub use crate::css_parser::*;
