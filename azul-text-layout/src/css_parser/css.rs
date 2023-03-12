//! High-level types and functions related to CSS parsing
pub use azul_simplecss::Error as CssSyntaxError;
use azul_simplecss::Tokenizer;
use std::collections::BTreeMap;
use std::{fmt, num::ParseIntError};

use crate::css::{
    CombinedCssPropertyType, Css, CssDeclaration, CssKeyMap, CssNthChildSelector,
    CssNthChildSelector::*, CssPath, CssPathPseudoSelector, CssPathSelector, CssPropertyType,
    CssRuleBlock, DynamicCssProperty, NodeTypeTag, NodeTypeTagParseError, Stylesheet,
};
use crate::css_parser;
pub use crate::css_parser::CssParsingError;

#[derive(Debug, Default, PartialEq, PartialOrd, Clone)]
#[repr(transparent)]
pub struct CssApiWrapper {
    pub css: Css,
}

/// Error that can happen during the parsing of a CSS value
#[derive(Debug, Clone, PartialEq)]
pub struct CssParseError<'a> {
    pub css_string: &'a str,
    pub error: CssParseErrorInner<'a>,
    pub location: (ErrorLocation, ErrorLocation),
}

impl<'a> CssParseError<'a> {
    /// Returns the string between the (start, end) location
    pub fn get_error_string(&self) -> &'a str {
        let (start, end) = (self.location.0.original_pos, self.location.1.original_pos);
        let s = &self.css_string[start..end];
        s.trim()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CssParseErrorInner<'a> {
    /// A hard error in the CSS syntax
    ParseError(CssSyntaxError),
    /// Error parsing dynamic CSS property, such as
    /// `#div { width: {{ my_id }} /* no default case */ }`
    DynamicCssParseError(DynamicCssParseError<'a>),
    /// Error while parsing a pseudo selector (like `:aldkfja`)
    PseudoSelectorParseError(CssPseudoSelectorParseError),
    /// The path has to be either `*`, `div`, `p` or something like that
    NodeTypeTag(NodeTypeTagParseError<'a>),
}

impl_display! { CssParseErrorInner<'a>, {
    ParseError(e) => format!("Parse Error: {:?}", e),
    UnclosedBlock => "Unclosed block",
    MalformedCss => "Malformed Css",
    DynamicCssParseError(e) => format!("{}", e),
    PseudoSelectorParseError(e) => format!("Failed to parse pseudo-selector: {}", e),
    NodeTypeTag(e) => format!("Failed to parse CSS selector path: {}", e),
}}

impl<'a> From<CssSyntaxError> for CssParseErrorInner<'a> {
    fn from(e: CssSyntaxError) -> Self {
        CssParseErrorInner::ParseError(e)
    }
}

impl_from! { DynamicCssParseError<'a>, CssParseErrorInner::DynamicCssParseError }
impl_from! { NodeTypeTagParseError<'a>, CssParseErrorInner::NodeTypeTag }

impl From<CssPseudoSelectorParseError> for CssParseErrorInner<'_> {
    fn from(value: CssPseudoSelectorParseError) -> Self {
        CssParseErrorInner::PseudoSelectorParseError(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CssPseudoSelectorParseError {
    InvalidNthChild(ParseIntError),
}

impl From<ParseIntError> for CssPseudoSelectorParseError {
    fn from(e: ParseIntError) -> Self {
        CssPseudoSelectorParseError::InvalidNthChild(e)
    }
}

impl_display! { CssPseudoSelectorParseError, {
    EmptyNthChild => format!("\
        Empty :nth-child() selector - nth-child() must at least take a number, \
        a pattern (such as \"2n+3\") or the values \"even\" or \"odd\"."
    ),
    InvalidNthChild(e) => format!("Invalid :nth-child pseudo-selector: ':{}'", e),
}}

/// Error that can happen during `css_parser::parse_key_value_pair`
#[derive(Debug, Clone, PartialEq)]
pub enum DynamicCssParseError<'a> {
    /// Unexpected value when parsing the string
    UnexpectedValue(CssParsingError<'a>),
}

impl_display! { DynamicCssParseError<'a>, {
    UnexpectedValue(e) => format!("{}", e),
}}

impl<'a> From<CssParsingError<'a>> for DynamicCssParseError<'a> {
    fn from(e: CssParsingError<'a>) -> Self {
        DynamicCssParseError::UnexpectedValue(e)
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ErrorLocation {
    pub original_pos: usize,
}

impl ErrorLocation {
    /// Given an error location, returns the (line, column)
    pub fn get_line_column_from_error(&self, css_string: &str) -> (usize, usize) {
        let error_location = self.original_pos.saturating_sub(1);
        let (mut line_number, mut total_characters) = (0, 0);

        for line in css_string[0..error_location].lines() {
            line_number += 1;
            total_characters += line.chars().count();
        }

        // Rust doesn't count "\n" as a character, so we have to add the line number count on top
        let total_characters = total_characters + line_number;
        let column_pos = error_location - total_characters.saturating_sub(2);

        (line_number, column_pos)
    }
}

impl<'a> fmt::Display for CssParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let start_location = self.location.0.get_line_column_from_error(self.css_string);
        let end_location = self.location.1.get_line_column_from_error(self.css_string);
        write!(
            f,
            "    start: line {}:{}\r\n    end: line {}:{}\r\n    text: \"{}\"\r\n    reason: {}",
            start_location.0,
            start_location.1,
            end_location.0,
            end_location.1,
            self.get_error_string(),
            self.error,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CssPathParseError<'a> {
    SyntaxError(CssSyntaxError),
    /// The path has to be either `*`, `div`, `p` or something like that
    NodeTypeTag(NodeTypeTagParseError<'a>),
    /// Error while parsing a pseudo selector (like `:aldkfja`)
    PseudoSelectorParseError(CssPseudoSelectorParseError),
}

impl_from! { NodeTypeTagParseError<'a>, CssPathParseError::NodeTypeTag }

impl From<CssPseudoSelectorParseError> for CssPathParseError<'_> {
    fn from(value: CssPseudoSelectorParseError) -> Self {
        CssPathParseError::PseudoSelectorParseError(value)
    }
}

impl<'a> From<CssSyntaxError> for CssPathParseError<'a> {
    fn from(e: CssSyntaxError) -> Self {
        CssPathParseError::SyntaxError(e)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnparsedCssRuleBlock<'a> {
    /// The css path (full selector) of the style ruleset
    pub path: CssPath,
    /// `"justify-content" => "center"`
    pub declarations: BTreeMap<&'a str, (&'a str, (ErrorLocation, ErrorLocation))>,
}
