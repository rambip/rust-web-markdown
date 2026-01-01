pub use pulldown_cmark::{CowStr, Options};
use pulldown_cmark::{Event, LinkType, Parser};

use core::ops::Range;
use std::collections::BTreeMap;

mod render;
use render::Renderer;

mod component;

pub struct ElementAttributes<H> {
    pub classes: Vec<String>,
    pub style: Option<String>,
    pub on_click: Option<H>,
}

impl<H> Default for ElementAttributes<H> {
    fn default() -> Self {
        Self {
            style: None,
            classes: vec![],
            on_click: None,
        }
    }
}

pub enum HtmlElement {
    Div,
    Span,
    Paragraph,
    BlockQuote,
    Ul,
    Ol(i32),
    Li,
    Heading(u8),
    Table,
    Thead,
    Trow,
    Tcell,
    Italics,
    Bold,
    StrikeThrough,
    Pre,
    Code,
}

pub struct StyleLink {
    pub rel: &'static str,
    pub href: &'static str,
    pub integrity: &'static str,
    pub crossorigin: &'static str,
}

pub const MATH_STYLE_SHEET_LINK: StyleLink = StyleLink {
    rel: "stylesheet",
    href: "https://cdn.jsdelivr.net/npm/katex@0.16.7/dist/katex.min.css",
    integrity: "sha384-3UiQGuEI4TTMaFmGIZumfRPtfKQ3trwQE2JgosJxCnGmQpL/lJdjpcHkaaFwHlcI",
    crossorigin: "anonymous",
};

pub trait Context<'a, 'callback>: 'a + Copy
where
    'callback: 'a,
{
    type View: Clone + 'callback;
    type Handler<T: 'callback>: 'callback;
    type MouseEvent: 'static;

    /// get all the properties from the context
    fn props(self) -> MarkdownProps;

    /// write the frontmatter (or metadata) string
    /// present at the top of the markdown source
    fn set_frontmatter(&mut self, frontmatter: String);

    fn render_links(self, link: LinkDescription<Self::View>) -> Result<Self::View, String>;

    /// calls a callback with the given input
    fn call_handler<T>(callback: &Self::Handler<T>, input: T);

    /// creates a callback that will fire when the user clicks on markdown
    fn make_md_handler(
        self,
        position: Range<usize>,
        stop_propagation: bool,
    ) -> Self::Handler<Self::MouseEvent>;

    #[cfg(feature = "debug")]
    fn send_debug_info(self, info: Vec<String>);

    /// creates a html element
    /// `attributes` contains the html attributes for this element
    fn el_with_attributes(
        self,
        e: HtmlElement,
        inside: Self::View,
        attributes: ElementAttributes<Self::Handler<Self::MouseEvent>>,
    ) -> Self::View;

    /// creates a html element, with default attributes
    fn el(self, e: HtmlElement, inside: Self::View) -> Self::View {
        self.el_with_attributes(e, inside, Default::default())
    }

    /// renders raw html, inside a span
    fn el_span_with_inner_html(
        self,
        inner_html: String,
        attributes: ElementAttributes<Self::Handler<Self::MouseEvent>>,
    ) -> Self::View;

    /// renders a `hr` element, with attributes
    fn el_hr(self, attributes: ElementAttributes<Self::Handler<Self::MouseEvent>>) -> Self::View;

    /// renders a `br` element
    fn el_br(self) -> Self::View;

    /// takes a vector of views and return a view
    fn el_fragment(self, children: Vec<Self::View>) -> Self::View;

    /// renders a link
    fn el_a(self, children: Self::View, href: String) -> Self::View;

    /// renders an image
    fn el_img(self, src: String, alt: String) -> Self::View;

    /// renders an empty view
    fn el_empty(self) -> Self::View {
        self.el_fragment(vec![])
    }

    /// renders raw text
    fn el_text(self, text: CowStr<'a>) -> Self::View;

    // renders a checkbox with attributes
    fn el_input_checkbox(
        self,
        checked: bool,
        attributes: ElementAttributes<Self::Handler<Self::MouseEvent>>,
    ) -> Self::View;

    fn has_custom_component(self, name: &str) -> bool;
    fn render_custom_component(
        self,
        name: &str,
        input: MdComponentProps<Self::View>,
    ) -> Result<Self::View, ComponentCreationError>;

    fn render_tasklist_marker(self, m: bool, position: Range<usize>) -> Self::View {
        let attributes = ElementAttributes {
            on_click: Some(self.make_md_handler(position, true)),
            ..Default::default()
        };
        self.el_input_checkbox(m, attributes)
    }

    fn render_rule(self, range: Range<usize>) -> Self::View {
        let attributes = ElementAttributes {
            on_click: Some(self.make_md_handler(range, false)),
            ..Default::default()
        };
        self.el_hr(attributes)
    }

    fn render_code(self, s: CowStr<'a>, range: Range<usize>) -> Self::View {
        let callback = self.make_md_handler(range.clone(), false);
        let attributes = ElementAttributes {
            on_click: Some(callback),
            ..Default::default()
        };
        self.el_with_attributes(HtmlElement::Code, self.el_text(s), attributes)
    }

    fn render_text(self, s: CowStr<'a>, range: Range<usize>) -> Self::View {
        let callback = self.make_md_handler(range, false);
        let attributes = ElementAttributes {
            on_click: Some(callback),
            ..Default::default()
        };
        self.el_with_attributes(HtmlElement::Span, self.el_text(s), attributes)
    }

    fn has_custom_links(self) -> bool;

    fn render_link(self, link: LinkDescription<Self::View>) -> Result<Self::View, String> {
        if self.has_custom_links() {
            self.render_links(link)
        } else {
            Ok(if link.image {
                self.el_img(link.url, link.title)
            } else {
                self.el_a(link.content, link.url)
            })
        }
    }
}

/// the description of a link, used to render it with a custom callback.
/// See [pulldown_cmark::Tag::Link] for documentation
pub struct LinkDescription<V> {
    /// the url of the link
    pub url: String,

    /// the html view of the element under the link
    pub content: V,

    /// the title of the link.
    /// If you don't know what it is, don't worry: it is ofter empty
    pub title: String,

    /// the type of link
    pub link_type: LinkType,

    /// wether the link is an image
    pub image: bool,
}

#[allow(dead_code)]
pub(crate) enum HtmlError {
    NotImplemented(String),
    Link(String),
    Syntax(String),
    CustomComponent { name: String, msg: String },
    Unavailable(String),
    Math,
}

impl ToString for HtmlError {
    fn to_string(&self) -> String {
        match self {
            HtmlError::Math => "invalid math".to_string(),
            HtmlError::NotImplemented(s) => format!("`{s}`: not implemented"),
            HtmlError::CustomComponent { name, msg } => {
                format!("Custom component `{name}` failed: `{msg}`")
            }
            HtmlError::Syntax(s) => format!("syntax error: {s}"),
            HtmlError::Link(s) => format!("invalid link: {s}"),
            HtmlError::Unavailable(s) => s.to_string(),
        }
    }
}

#[derive(PartialEq)]
/// the arguments given to a markdown component.
/// `attributes`: a map of (attribute_name, attribute_value) pairs
/// `children`: the interior markdown of the component
///
/// For example,
/// ```md
/// <MyBox color="blue" size="5">
///
/// **hey !**
///
/// </MyBox>
/// ```
///
/// Will be translated to
/// ```rust,ignore
/// MdComponentProps {
///     attributes: BTreeMap::from([("color", "blue"), ("size", "5")]),
///     children: ... // html view of **hey**
/// }
/// ```
pub struct MdComponentProps<V> {
    attributes: BTreeMap<String, MdComponentAttribute>,
    pub children: V,
}

#[non_exhaustive]
#[derive(PartialEq, Clone)]
pub struct MdComponentAttribute {
    pub value: String,
    /// Location in input markdown `&str` which contains the `attributes_value`.
    pub range: Range<usize>,
}

impl<V> MdComponentProps<V> {
    /// returns the attribute string corresponding to the key `name`.
    /// returns None if the attribute was not provided
    pub fn get(&self, name: &str) -> Option<String> {
        Self::get_attribute(self, name).map(|a| a.value)
    }

    pub fn get_attribute(&self, name: &str) -> Option<MdComponentAttribute> {
        self.attributes.get(name).cloned()
    }

    /// returns the attribute corresponding to the key `name`, once parsed.
    /// If the attribute doesn't exist or if the parsing fail, returns an error.
    pub fn get_parsed<T>(&self, name: &str) -> Result<T, String>
    where
        T: std::str::FromStr,
        T::Err: core::fmt::Debug,
    {
        match self.get(name) {
            Some(x) => x.clone().parse().map_err(|e| format!("{e:?}")),
            None => Err(format!("please provide the attribute `{name}`")),
        }
    }

    /// same thing as `get_parsed`, but if the attribute doesn't exist,
    /// return None
    pub fn get_parsed_optional<T>(&self, name: &str) -> Result<Option<T>, String>
    where
        T: std::str::FromStr,
        T::Err: core::fmt::Debug,
    {
        match self.get(name) {
            Some(x) => match x.parse() {
                Ok(a) => Ok(Some(a)),
                Err(e) => Err(format!("{e:?}")),
            },
            None => Ok(None),
        }
    }
}

/// Error raised by the user of the library,
/// when creating a component.
/// It is automatically converted from any type of error.
pub struct ComponentCreationError(String);

impl<T: std::fmt::Debug> From<T> for ComponentCreationError {
    fn from(value: T) -> Self {
        Self(format!("{:?}", value))
    }
}

pub struct MarkdownProps {
    pub hard_line_breaks: bool,

    pub wikilinks: bool,

    pub parse_options: Option<pulldown_cmark::Options>,

    pub theme: Option<&'static str>,
}

pub fn markdown_component<'a, 'callback, F: Context<'a, 'callback>>(
    cx: F,
    source: &'a str,
) -> F::View {
    let parse_options_default = Options::ENABLE_GFM
        | Options::ENABLE_MATH
        | Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_WIKILINKS
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS;
    let options = cx.props().parse_options.unwrap_or(parse_options_default);
    let mut stream: Vec<_> = Parser::new_ext(source, options)
        .into_offset_iter()
        .collect();

    #[cfg(feature = "debug")]
    {
        let debug_info: Vec<String> = stream.iter().map(|x| format!("{:?}", x)).collect();
        cx.send_debug_info(debug_info)
    }

    if cx.props().hard_line_breaks {
        for (r, _) in &mut stream {
            if *r == Event::SoftBreak {
                *r = Event::HardBreak
            }
        }
    }

    let elements = Renderer::new(cx, &mut stream.into_iter()).collect::<Vec<_>>();

    cx.el_fragment(elements)
}

/// Gives the byte range of `parent` that is made up of `inner`.
/// Once it is stable, this should be replaced by https://github.com/rust-lang/rust/issues/126769.
pub(crate) fn get_substr_range(parent: &str, inner: &str) -> Option<Range<usize>> {
    let a = parent.as_ptr().addr();
    let b = inner.as_ptr().addr();

    if b < a {
        return None;
    }
    let start_of_inner_in_parent = b - a;

    // If parent starts at address 0, and inner ends at address usize::MAX, this could overflow.
    // In practice this should be impossible for valid inputs, and impossible on all realistic systems, but validate it for completeness and clarity.
    let one_past_end_of_inner_in_parent =
        start_of_inner_in_parent.checked_add(inner.len()).unwrap();

    if one_past_end_of_inner_in_parent > parent.len() {
        return None;
    }

    Some((start_of_inner_in_parent)..(start_of_inner_in_parent + inner.len()))
}

pub(crate) fn offset_range(range: Range<usize>, shift: usize) -> Range<usize> {
    return (range.start + shift)..(range.end.checked_add(shift).unwrap());
}

#[cfg(test)]
mod test {
    use super::*;

    fn check_range(s: &str, range: Range<usize>) {
        assert_eq!(get_substr_range(s, &s[range.clone()]).unwrap(), range)
    }

    #[test]
    fn get_substring_range_valid() {
        let s = "<abc>";
        check_range(s, 0..1);
        check_range(s, 0..5);
        check_range(s, 4..5);
        assert_eq!(get_substr_range(s, s).unwrap(), 0..5);
    }

    #[test]
    fn get_substring_range_invalid() {
        let s = "<adc>";
        let s2 = "<abc>";
        assert_eq!(get_substr_range(s, s2), None);
        assert_eq!(get_substr_range(s2, s), None);
        assert_eq!(get_substr_range(s, &s2[0..1]), None);
    }

    #[test]
    fn get_substring_range_empty() {
        let s = "<adc>";
        check_range(s, 1..1);
    }
}
