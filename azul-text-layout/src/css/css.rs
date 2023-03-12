//! Types and methods used to describe the style of an application
use crate::css::css_properties::{CssProperty, CssPropertyType};
use core::fmt;

/// Css stylesheet - contains a parsed CSS stylesheet in "rule blocks",
/// i.e. blocks of key-value pairs associated with a selector path.
#[derive(Debug, Default, PartialEq, PartialOrd, Clone)]
#[repr(C)]
pub struct Css {
    /// One CSS stylesheet can hold more than one sub-stylesheet:
    /// For example, when overriding native styles, the `.sort_by_specificy()` function
    /// should not mix the two stylesheets during sorting.
    pub stylesheets: Vec<Stylesheet>,
}

impl Css {}

#[derive(Debug, Default, PartialEq, PartialOrd, Clone)]
#[repr(C)]
pub struct Stylesheet {
    /// The style rules making up the document - for example, de-duplicated CSS rules
    pub rules: Vec<CssRuleBlock>,
}

impl Stylesheet {}

impl From<Vec<CssRuleBlock>> for Stylesheet {
    fn from(rules: Vec<CssRuleBlock>) -> Self {
        Self {
            rules: rules.into(),
        }
    }
}

/// Contains one parsed `key: value` pair, static or dynamic
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CssDeclaration {}

impl CssDeclaration {}

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

/// One block of rules that applies a bunch of rules to a "path" in the style, i.e.
/// `div#myid.myclass -> { ("justify-content", "center") }`
#[derive(Debug, Clone, PartialOrd, PartialEq)]
#[repr(C)]
pub struct CssRuleBlock {
    /// The css path (full selector) of the style ruleset
    pub path: CssPath,
    /// `"justify-content: center"` =>
    /// `CssDeclaration::Static(CssProperty::JustifyContent(LayoutJustifyContent::Center))`
    pub declarations: Vec<CssDeclaration>,
}

/// Signifies the type (i.e. the discriminant value) of a DOM node
/// without carrying any of its associated data
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum NodeTypeTag {
    Body,
    Div,
    Br,
    P,
    Img,
    IFrame,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeTypeTagParseError<'a> {
    Invalid(&'a str),
}

impl<'a> fmt::Display for NodeTypeTagParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            NodeTypeTagParseError::Invalid(e) => write!(f, "Invalid node type: {}", e),
        }
    }
}

impl fmt::Display for NodeTypeTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NodeTypeTag::Body => write!(f, "body"),
            NodeTypeTag::Div => write!(f, "div"),
            NodeTypeTag::Br => write!(f, "br"),
            NodeTypeTag::P => write!(f, "p"),
            NodeTypeTag::Img => write!(f, "img"),
            NodeTypeTag::IFrame => write!(f, "iframe"),
        }
    }
}

/// Represents a full CSS path (i.e. the "div#id.class" selector belonging to
///  a CSS "content group" (the following key-value block)).
///
/// ```no_run,ignore
/// "#div > .my_class:focus" ==
/// [
///   CssPathSelector::Type(NodeTypeTag::Div),
///   CssPathSelector::PseudoSelector(CssPathPseudoSelector::LimitChildren),
///   CssPathSelector::Class("my_class"),
///   CssPathSelector::PseudoSelector(CssPathPseudoSelector::Focus),
/// ]
#[derive(Clone, Hash, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct CssPath {
    pub selectors: Vec<CssPathSelector>,
}

impl CssPath {}

impl fmt::Display for CssPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for selector in self.selectors.iter() {
            write!(f, "{}", selector)?;
        }
        Ok(())
    }
}

impl fmt::Debug for CssPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(C, u8)]
pub enum CssPathSelector {
    /// Represents the `*` selector
    Global,
    /// `div`, `p`, etc.
    Type(NodeTypeTag),
    /// `.something`
    Class(String),
    /// `#something`
    Id(String),
    /// `:something`
    PseudoSelector(CssPathPseudoSelector),
    /// Represents the `>` selector
    DirectChildren,
    /// Represents the ` ` selector
    Children,
}

impl Default for CssPathSelector {
    fn default() -> Self {
        CssPathSelector::Global
    }
}

impl fmt::Display for CssPathSelector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::CssPathSelector::*;
        match &self {
            Global => write!(f, "*"),
            Type(n) => write!(f, "{}", n),
            Class(c) => write!(f, ".{}", c),
            Id(i) => write!(f, "#{}", i),
            PseudoSelector(p) => write!(f, ":{}", p),
            DirectChildren => write!(f, ">"),
            Children => write!(f, " "),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(C, u8)]
pub enum CssPathPseudoSelector {
    /// `:first`
    First,
    /// `:last`
    Last,
    /// `:nth-child`
    NthChild(CssNthChildSelector),
    /// `:hover` - mouse is over element
    Hover,
    /// `:active` - mouse is pressed and over element
    Active,
    /// `:focus` - element has received focus
    Focus,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(C, u8)]
pub enum CssNthChildSelector {
    Number(u32),
    Even,
    Odd,
    Pattern(CssNthChildPattern),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(C)]
pub struct CssNthChildPattern {
    pub repeat: u32,
    pub offset: u32,
}

impl fmt::Display for CssNthChildSelector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::CssNthChildSelector::*;
        match &self {
            Number(u) => write!(f, "{}", u),
            Even => write!(f, "even"),
            Odd => write!(f, "odd"),
            Pattern(p) => write!(f, "{}n + {}", p.repeat, p.offset),
        }
    }
}

impl fmt::Display for CssPathPseudoSelector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::CssPathPseudoSelector::*;
        match &self {
            First => write!(f, "first"),
            Last => write!(f, "last"),
            NthChild(u) => write!(f, "nth-child({})", u),
            Hover => write!(f, "hover"),
            Active => write!(f, "active"),
            Focus => write!(f, "focus"),
        }
    }
}

pub struct RuleIterator<'a> {
    current_stylesheet: usize,
    current_rule: usize,
    css: &'a Css,
}

impl<'a> Iterator for RuleIterator<'a> {
    type Item = &'a CssRuleBlock;
    fn next(&mut self) -> Option<&'a CssRuleBlock> {
        let current_stylesheet = self.css.stylesheets.get(self.current_stylesheet)?;
        match current_stylesheet.rules.get(self.current_rule) {
            Some(s) => {
                self.current_rule += 1;
                Some(s)
            }
            None => {
                self.current_rule = 0;
                self.current_stylesheet += 1;
                self.next()
            }
        }
    }
}
