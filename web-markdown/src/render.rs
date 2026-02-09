use core::iter::Peekable;
use core::ops::Range;
use pulldown_cmark::CowStr;

use core::marker::PhantomData;
use std::collections::BTreeMap;

use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

use pulldown_cmark::{Alignment, CodeBlockKind, Event, Tag, TagEnd};

#[derive(Eq, PartialEq)]
enum MathMode {
    Inline,
    Display,
}

use super::HtmlElement::*;
use super::{Context, ElementAttributes, HtmlError, LinkDescription, MdComponentProps};

use crate::component::{ComponentCall, CustomHtmlTag, CustomHtmlTagError};
use crate::{get_substr_range, offset_range, MdComponentAttribute};

// load the default syntect options to highlight code
lazy_static::lazy_static! {
    static ref SYNTAX_SET: SyntaxSet = {
        SyntaxSet::load_defaults_newlines()
    };
    static ref THEME_SET: ThemeSet = {
        ThemeSet::load_defaults()
    };
}

impl HtmlError {
    fn not_implemented(message: impl ToString) -> Self {
        HtmlError::NotImplemented(message.to_string())
    }
    fn syntax(message: impl ToString) -> Self {
        HtmlError::Syntax(message.to_string())
    }
    fn component(name: impl ToString, msg: impl ToString) -> Self {
        HtmlError::CustomComponent {
            name: name.to_string(),
            msg: msg.to_string(),
        }
    }
}

/// `highlight_code(content, ss, ts)` render the content `content`
/// with syntax highlighting
fn highlight_code(theme_name: Option<&str>, content: &str, kind: &CodeBlockKind) -> Option<String> {
    let lang = match kind {
        CodeBlockKind::Fenced(x) => x,
        CodeBlockKind::Indented => return None,
    };

    let theme_name = theme_name.unwrap_or("base16-ocean.light");
    let theme = THEME_SET
        .themes
        .get(theme_name)
        .expect("unknown theme")
        .clone();

    syntect::html::highlighted_html_for_string(
        content,
        &SYNTAX_SET,
        SYNTAX_SET.find_syntax_by_token(lang)?,
        &theme,
    )
    .ok()
}

/// renders a source code in a code block, with syntax highlighting if possible.
/// `cx`: the current markdown context
/// `source`: the source to render
/// `range`: the position of the code in the original source
fn render_code_block<'a, 'callback, F: Context<'a, 'callback>>(
    cx: F,
    source: String,
    k: &CodeBlockKind,
    range: Range<usize>,
) -> F::View {
    let code_attributes = ElementAttributes {
        on_click: Some(cx.make_md_handler(range, true)),
        ..Default::default()
    };

    match highlight_code(cx.props().theme, &source, k) {
        None => cx.el_with_attributes(
            Code,
            cx.el(Code, cx.el_text(source.into())),
            code_attributes,
        ),
        Some(x) => cx.el_span_with_inner_html(x, code_attributes),
    }
}

#[cfg(feature = "maths")]
/// `render_maths(content)` returns a html node
/// with the latex content `content` compiled inside
fn render_maths<'a, 'callback, F: Context<'a, 'callback>>(
    cx: F,
    content: &str,
    display_mode: MathMode,
    range: Range<usize>,
) -> Result<F::View, HtmlError> {
    use katex::{KatexContext, Settings};

    // The context caches fonts, macros, and environments – reuse it between renders.
    let ctx = KatexContext::default();

    // Start with the default configuration and tweak as needed.
    let settings = Settings::builder()
        .display_mode(display_mode == MathMode::Display)
        .build();

    let class_name = match display_mode {
        MathMode::Inline => "math-inline",
        MathMode::Display => "math-flow",
    };

    let callback = cx.make_md_handler(range, true);

    let attributes = ElementAttributes {
        classes: vec![class_name.to_string()],
        on_click: Some(callback),
        ..Default::default()
    };

    match katex::render_to_string(&ctx, content, &settings) {
        Ok(x) => Ok(cx.el_span_with_inner_html(x, attributes)),
        Err(_) => Err(HtmlError::Math),
    }
}
#[cfg(not(feature = "maths"))]
fn render_maths<'a, 'callback, F: Context<'a, 'callback>>(
    _cx: F,
    _content: &str,
    _display_mode: MathMode,
    _range: Range<usize>,
) -> Result<F::View, HtmlError> {
    Err(HtmlError::Unavailable(
        "Math was not enabled during compilation of the library. Please unable the `maths` feature"
            .into(),
    ))
}

/// `align_string(align)` gives the css string
/// that is used to align text according to `align`
fn align_string(align: Alignment) -> &'static str {
    match align {
        Alignment::Left => "text-align: left",
        Alignment::Right => "text-align: right",
        Alignment::Center => "text-align: center",
        Alignment::None => "",
    }
}

#[derive(Debug)]
pub struct RenderEvent<'a> {
    event: Event<'a>,
    custom_tag: Option<CowStr<'a>>,
    range: Range<usize>,
}

/// Manage the creation of a [`F::View`]
/// from a stream of markdown events
pub struct Renderer<'a, 'callback, 'c, I, F>
where
    I: Iterator<Item = (Event<'a>, Range<usize>)>,
    'callback: 'a,
    F: Context<'a, 'callback>,
{
    __marker: PhantomData<&'callback ()>,
    /// the markdown context
    cx: F,
    /// the stream of markdown [`Event`]s
    stream: &'c mut Peekable<I>,
    /// the alignment settings inside the current table
    column_alignment: Option<Vec<Alignment>>,
    /// the current horizontal index of the cell we are in.
    /// TODO: remove it
    cell_index: usize,
    /// the root tag that this renderer is rendering
    end_tag: Option<TagEnd>,
    /// the current component we are inside of.
    /// custom components doesn't allow nesting.
    current_component: Option<String>,
}

/// Returns true if `raw_html` appears to be a custom component tag.
///
/// A valid custom component tag must:
/// - Start with '<'
/// - End with '>'
/// - Have a tag name that either:
///   - Starts with an uppercase letter (A-Z), OR
///   - Starts with a lowercase letter (a-z) AND contains at least one dash (-)
///
/// This validation prevents standard HTML tags like `<div>`, `<span>`, `<p>` from being
/// treated as custom components while allowing custom component names like:
/// - `<MyComponent>` (uppercase start, no dash needed)
/// - `<my-component>` (lowercase start, has dash)
/// - `<My-Component>` (uppercase start, has dash)
///
/// The function also handles:
/// - Self-closing tags: `<My-Component/>`
/// - Tags with attributes: `<My-Component attr="value">`
/// - Closing tags: `</My-Component>`
fn can_be_custom_component(raw_html: &str) -> bool {
    let s = raw_html.trim();
    if s.len() < 3 {
        return false;
    }

    let chars: Vec<_> = s.chars().collect();

    // Must start with '<' and end with '>'
    if chars[0] != '<' || chars[chars.len() - 1] != '>' {
        return false;
    }

    // Extract tag name: skip '<' and optionally '/' for closing tags
    let mut idx = 1;
    if idx < chars.len() && chars[idx] == '/' {
        idx += 1;
    }

    if idx >= chars.len() {
        return false;
    }

    // Parse tag name until we hit whitespace, '/', or '>'
    let mut tag_name = String::new();
    while idx < chars.len() {
        let c = chars[idx];
        if c.is_whitespace() || c == '/' || c == '>' {
            break;
        }
        tag_name.push(c);
        idx += 1;
    }

    if tag_name.is_empty() {
        return false;
    }

    // Check if tag name is valid for a custom component
    let first_char = tag_name.chars().next().unwrap();
    let has_dash = tag_name.contains('-');

    // Valid if:
    // - Starts with uppercase letter (A-Z), OR
    // - Starts with lowercase letter (a-z) AND contains a dash
    if first_char.is_ascii_uppercase() {
        // Uppercase start is always valid (e.g., MyComponent, My-Component)
        true
    } else if first_char.is_ascii_lowercase() && has_dash {
        // Lowercase start is only valid if it has a dash (e.g., my-component)
        true
    } else {
        // Otherwise not a custom component (e.g., div, span, p)
        false
    }
}

impl<'a, 'callback, 'c, I, F> Iterator for Renderer<'a, 'callback, 'c, I, F>
where
    I: Iterator<Item = (Event<'a>, Range<usize>)>,
    'callback: 'a,
    F: Context<'a, 'callback>,
{
    type Item = F::View;

    fn next(&mut self) -> Option<Self::Item> {
        use Event::*;
        let cx = self.cx;
        let render_event = self.next_render_event()?;
        let range = render_event.range.clone();

        let rendered = if let Some(raw_html) = render_event.custom_tag {
            let custom_tag = CustomHtmlTag::from_str(&raw_html, range.start);
            match (custom_tag, &self.current_component) {
                (Ok(CustomHtmlTag::Inline(s)), None) => self.custom_component_inline(s),
                (Ok(CustomHtmlTag::End(s)), None) => {
                    Err(HtmlError::component(s, "expected start, not end"))
                }
                (Ok(CustomHtmlTag::Start(s)), None) => self.custom_component(s),
                (
                    Err(CustomHtmlTagError {
                        name: Some(name),
                        message,
                    }),
                    _,
                ) => Err(HtmlError::component(
                    name,
                    format!("not a valid component: {message}"),
                )),
                (
                    Err(CustomHtmlTagError {
                        name: None,
                        message: _,
                    }),
                    _,
                ) => Ok(self.html(&raw_html)),
                (Ok(CustomHtmlTag::End(s)), Some(x)) if s == x => return None,
                _ => Err(HtmlError::component("?", "invalid component")),
            }
        } else {
            match render_event.event {
                Start(t) => self.render_tag(t, range),
                End(end) => {
                    // check if the closing tag is the tag that was open
                    // when this renderer was created
                    match self.end_tag {
                        Some(t) if t == end => return None,
                        Some(t) => panic!("{end:?} is a wrong closing tag, expected {t:?}"),
                        None => panic!("didn't expect a closing tag"),
                    }
                }
                Text(s) => Ok(cx.render_text(s, range)),
                Code(s) => Ok(cx.render_code(s, range)),
                InlineHtml(s) => Ok(self.html(&s)),
                Html(raw_html) => Ok(self.html(&raw_html)),
                FootnoteReference(_) => Err(HtmlError::not_implemented("footnotes refs")),
                SoftBreak => Ok(cx.el_text(" ".into())),
                HardBreak => Ok(self.cx.el_br()),
                Rule => Ok(cx.render_rule(range)),
                TaskListMarker(m) => Ok(cx.render_tasklist_marker(m, range)),
                InlineMath(content) => render_maths(self.cx, &content, MathMode::Inline, range),
                DisplayMath(content) => render_maths(self.cx, &content, MathMode::Display, range),
                _ => panic!(),
            }
        };

        Some(rendered.unwrap_or_else(|e: HtmlError| {
            self.cx.el_with_attributes(
                Span,
                self.cx
                    .el_fragment(vec![self.cx.el_text(e.to_string().into()), self.cx.el_br()]),
                ElementAttributes {
                    classes: vec!["markdown-error".to_string()],
                    on_click: None,
                    ..Default::default()
                },
            )
        }))
    }
}

impl<'a, 'callback, 'c, I, F> Renderer<'a, 'callback, 'c, I, F>
where
    I: Iterator<Item = (Event<'a>, Range<usize>)>,
    F: Context<'a, 'callback>,
{
    /// creates a new renderer from a stream of events.
    /// It returns an iterator of [`F::View`]
    pub fn new(cx: F, events: &'c mut Peekable<I>) -> Self {
        Self {
            __marker: PhantomData,
            cx,
            stream: events,
            column_alignment: None,
            cell_index: 0,
            end_tag: None,
            current_component: None,
        }
    }

    fn next_render_event(&mut self) -> Option<RenderEvent<'a>> {
        let (item, range): (Event<'a>, Range<usize>) = self.stream.next()?;
        let custom_tag = if let Event::Start(Tag::HtmlBlock) = item {
            let maybe_inside_event = self.stream.next_if(|(x, _)| match x {
                Event::Html(raw_html) if can_be_custom_component(raw_html) => true,
                _ => false,
            });
            match maybe_inside_event {
                Some((Event::Html(raw_html), r)) => {
                    match CustomHtmlTag::from_str(&raw_html, r.start) {
                        Ok(CustomHtmlTag::Start(x)) if self.cx.has_custom_component(x.name) => {
                            Some(raw_html)
                        }
                        Ok(CustomHtmlTag::End(name)) if self.cx.has_custom_component(name) => {
                            Some(raw_html)
                        }
                        Ok(CustomHtmlTag::Inline(x)) if self.cx.has_custom_component(x.name) => {
                            Some(raw_html)
                        }
                        _ => None,
                    }
                }
                _ => None,
            }
        } else {
            None
        };
        if custom_tag.is_some() {
            assert!(matches!(
                self.stream.next(),
                Some((Event::End(TagEnd::HtmlBlock), _))
            ));
            Some(RenderEvent {
                event: item,
                custom_tag,
                range: range,
            })
        } else {
            Some(match item {
                Event::InlineHtml(ref x) => RenderEvent {
                    // FIXME: avoid clone
                    custom_tag: Some(x.clone()),
                    event: item,
                    range,
                },
                _ => RenderEvent {
                    event: item,
                    custom_tag: None,
                    range,
                },
            })
        }
    }

    fn html(&mut self, raw_html: &str) -> F::View {
        self.cx
            .el_span_with_inner_html(raw_html.to_string(), Default::default())
    }

    /// Convert attributes from [ComponentCall] format to [MdComponentProps] format.
    fn convert_attributes(input: ComponentCall) -> BTreeMap<String, MdComponentAttribute> {
        // TODO: this should probably unescape the attribute values.
        BTreeMap::from_iter(input.attributes.iter().map(|(k, v)| {
            (
                k.to_string(),
                MdComponentAttribute {
                    value: v.to_string(),
                    range: offset_range(
                        get_substr_range(input.full_string, v).unwrap(),
                        input.range_offset,
                    ),
                },
            )
        }))
    }

    /// Renders a custom component with children.
    fn custom_component(&mut self, description: ComponentCall) -> Result<F::View, HtmlError> {
        let name: &str = description.name;
        if !self.cx.has_custom_component(name) {
            return Err(HtmlError::component(name, "not a valid component"));
        }

        let sub_renderer = Renderer {
            __marker: PhantomData,
            cx: self.cx,
            stream: self.stream,
            column_alignment: self.column_alignment.clone(),
            cell_index: 0,
            end_tag: self.end_tag,
            current_component: Some(description.name.to_string()),
        };
        let children = self.cx.el_fragment(sub_renderer.collect());

        let props = MdComponentProps {
            attributes: Self::convert_attributes(description),
            children,
        };

        match self.cx.render_custom_component(name, props) {
            Ok(x) => Ok(x),
            Err(e) => Err(HtmlError::CustomComponent {
                name: name.to_string(),
                msg: e.0,
            }),
        }
    }

    /// Renders a custom component without children.
    fn custom_component_inline(
        &mut self,
        description: ComponentCall,
    ) -> Result<F::View, HtmlError> {
        let name: &str = description.name;

        let props = MdComponentProps {
            attributes: Self::convert_attributes(description),
            children: self.cx.el_empty(),
        };

        match self.cx.render_custom_component(name, props) {
            Ok(x) => Ok(x),
            Err(e) => Err(HtmlError::CustomComponent {
                name: name.to_string(),
                msg: e.0,
            }),
        }
    }

    /// renders events in a new renderer,
    /// recursively, until the end of the tag
    fn children(&mut self, tag: Tag<'a>) -> F::View {
        let sub_renderer = Renderer {
            __marker: PhantomData,
            cx: self.cx,
            stream: self.stream,
            column_alignment: self.column_alignment.clone(),
            cell_index: 0,
            end_tag: Some(tag.to_end()),
            current_component: self.current_component.clone(),
        };
        self.cx.el_fragment(sub_renderer.collect())
    }

    /// Collect all text inside `tag` until its closing `End` event.
    ///
    /// Older versions assumed there was exactly one `Event::Text` followed
    /// by the closing `Event::End`, which is not guaranteed with
    /// pulldown-cmark 0.13+ (empty or multi-chunk blocks). This version
    /// accumulates text and stops at the matching closing tag instead
    /// of panicking.
    fn children_text(&mut self, tag: Tag<'a>) -> Option<String> {
        let end = tag.to_end();
        let mut buf = String::new();

        for (event, _range) in self.stream.by_ref() {
            match event {
                pulldown_cmark::Event::Text(s) => buf.push_str(&s),
                pulldown_cmark::Event::SoftBreak | pulldown_cmark::Event::HardBreak => {
                    // Represent line breaks explicitly in the collected text
                    buf.push('\n');
                }
                pulldown_cmark::Event::End(e) if e == end => {
                    // We reached the closing tag for this block
                    return if buf.is_empty() { None } else { Some(buf) };
                }
                // These shouldn't normally appear inside code/metadata blocks,
                // but if they do, panic so this code can be updated to accommodate them.
                _ => panic!("Unexpected content inside of children_text emitted by pulldown-cmark"),
            }
        }

        // If we run out of events without seeing the closing tag,
        // just return whatever we collected (or None if empty).
        if buf.is_empty() {
            None
        } else {
            Some(buf)
        }
    }

    fn render_tag(&mut self, tag: Tag<'a>, range: Range<usize>) -> Result<F::View, HtmlError> {
        let mut cx = self.cx;
        Ok(match tag.clone() {
            Tag::HtmlBlock => self.children(tag),
            Tag::Paragraph => cx.el(Paragraph, self.children(tag)),
            Tag::Heading { level, .. } => cx.el(Heading(level as u8), self.children(tag)),
            Tag::BlockQuote(_) => cx.el(BlockQuote, self.children(tag)),
            Tag::CodeBlock(k) => {
                render_code_block(cx, self.children_text(tag).unwrap_or_default(), &k, range)
            }
            Tag::List(Some(n0)) => cx.el(Ol(n0 as i32), self.children(tag)),
            Tag::List(None) => cx.el(Ul, self.children(tag)),
            Tag::Item => cx.el(Li, self.children(tag)),
            Tag::Table(align) => {
                self.column_alignment = Some(align);
                cx.el(Table, self.children(tag))
            }
            Tag::TableHead => cx.el(Thead, self.children(tag)),
            Tag::TableRow => cx.el(Trow, self.children(tag)),
            Tag::TableCell => {
                let align = self.column_alignment.clone().unwrap()[self.cell_index];
                self.cell_index += 1;
                cx.el_with_attributes(
                    Tcell,
                    self.children(tag),
                    ElementAttributes {
                        style: Some(align_string(align).to_string()),
                        ..Default::default()
                    },
                )
            }
            Tag::Emphasis => cx.el(Italics, self.children(tag)),
            Tag::Strong => cx.el(Bold, self.children(tag)),
            Tag::Strikethrough => cx.el(StrikeThrough, self.children(tag)),
            Tag::Image {
                link_type,
                dest_url,
                title,
                ..
            } => {
                let description = LinkDescription {
                    url: dest_url.to_string(),
                    title: title.to_string(),
                    content: self.children(tag),
                    link_type,
                    image: true,
                };
                cx.render_link(description).map_err(HtmlError::Link)?
            }
            Tag::Link {
                link_type,
                dest_url,
                title,
                ..
            } => {
                let description = LinkDescription {
                    url: dest_url.to_string(),
                    title: title.to_string(),
                    content: self.children(tag),
                    link_type,
                    image: false,
                };
                cx.render_link(description).map_err(HtmlError::Link)?
            }
            Tag::FootnoteDefinition(_) => {
                return Err(HtmlError::not_implemented("footnote not implemented"))
            }
            Tag::MetadataBlock { .. } => {
                if let Some(text) = self.children_text(tag) {
                    cx.set_frontmatter(text)
                }
                cx.el_empty()
            }
            Tag::DefinitionList => {
                return Err(HtmlError::not_implemented(
                    "definition list not implemented",
                ))
            }
            Tag::DefinitionListTitle => {
                return Err(HtmlError::not_implemented(
                    "definition list not implemented",
                ))
            }
            Tag::DefinitionListDefinition => {
                return Err(HtmlError::not_implemented(
                    "definition list not implemented",
                ))
            }
            Tag::Superscript => {
                return Err(HtmlError::not_implemented("superscript not implemented"))
            }
            Tag::Subscript => return Err(HtmlError::not_implemented("subscript not implemented")),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_be_custom_component_uppercase_start() {
        // Uppercase start should always be valid
        assert!(can_be_custom_component("<MyComponent>"));
        assert!(can_be_custom_component("<Counter>"));
        assert!(can_be_custom_component("<DataTable>"));
        assert!(can_be_custom_component("<MyComponent/>"));
        assert!(can_be_custom_component("</MyComponent>"));
        assert!(can_be_custom_component("<MyComponent attr=\"value\">"));
        assert!(can_be_custom_component("<My-Component>"));
        assert!(can_be_custom_component("<MY-COMPONENT>"));
    }

    #[test]
    fn test_can_be_custom_component_lowercase_with_dash() {
        // Lowercase start with dash should be valid
        assert!(can_be_custom_component("<my-component>"));
        assert!(can_be_custom_component("<data-table>"));
        assert!(can_be_custom_component("<custom-counter>"));
        assert!(can_be_custom_component("<my-component/>"));
        assert!(can_be_custom_component("</my-component>"));
        assert!(can_be_custom_component("<my-component attr=\"value\">"));
        assert!(can_be_custom_component("<a-b>"));
        assert!(can_be_custom_component("<my-custom-widget>"));
    }

    #[test]
    fn test_can_be_custom_component_lowercase_no_dash() {
        // Lowercase start without dash should be invalid (standard HTML tags)
        assert!(!can_be_custom_component("<div>"));
        assert!(!can_be_custom_component("<span>"));
        assert!(!can_be_custom_component("<p>"));
        assert!(!can_be_custom_component("<section>"));
        assert!(!can_be_custom_component("<article>"));
        assert!(!can_be_custom_component("<header>"));
        assert!(!can_be_custom_component("<footer>"));
        assert!(!can_be_custom_component("<div/>"));
        assert!(!can_be_custom_component("</div>"));
        assert!(!can_be_custom_component("<div class=\"test\">"));
    }

    #[test]
    fn test_can_be_custom_component_edge_cases() {
        // Empty or invalid tags
        assert!(!can_be_custom_component("<>"));
        assert!(!can_be_custom_component("</>"));
        assert!(!can_be_custom_component(""));
        assert!(!can_be_custom_component("< >"));
        assert!(!can_be_custom_component("text"));

        // Missing brackets
        assert!(!can_be_custom_component("MyComponent>"));
        assert!(!can_be_custom_component("<MyComponent"));

        // Whitespace handling
        assert!(can_be_custom_component("  <MyComponent>  "));
        assert!(can_be_custom_component("  <my-component>  "));
        assert!(!can_be_custom_component("  <div>  "));
    }

    #[test]
    fn test_can_be_custom_component_with_attributes() {
        // With attributes
        assert!(can_be_custom_component(
            "<Counter initial=\"5\" step=\"1\"/>"
        ));
        assert!(can_be_custom_component(
            "<my-widget data=\"test\" class=\"styled\"/>"
        ));
        assert!(!can_be_custom_component(
            "<div class=\"container\" id=\"main\">"
        ));
    }

    #[test]
    fn test_can_be_custom_component_self_closing() {
        // Self-closing tags
        assert!(can_be_custom_component("<MyComponent/>"));
        assert!(can_be_custom_component("<my-component/>"));
        assert!(!can_be_custom_component("<div/>"));
        assert!(!can_be_custom_component("<span/>"));
    }

    #[test]
    fn test_can_be_custom_component_closing_tags() {
        // Closing tags
        assert!(can_be_custom_component("</MyComponent>"));
        assert!(can_be_custom_component("</my-component>"));
        assert!(!can_be_custom_component("</div>"));
        assert!(!can_be_custom_component("</span>"));
    }
}
