use core::ops::Range;

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

#[cfg(feature = "maths")]
use katex;

use super::HtmlElement::*;
use super::{Context, ElementAttributes, HtmlError, LinkDescription, MdComponentProps};

use crate::component::{ComponentCall, CustomHtmlTag, CustomHtmlTagError};
use crate::MdComponentAttribute;

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

    let theme_name = theme_name.clone().unwrap_or("base16-ocean.light");
    let theme = THEME_SET
        .themes
        .get(theme_name)
        .expect("unknown theme")
        .clone();

    Some(
        syntect::html::highlighted_html_for_string(
            content,
            &SYNTAX_SET,
            SYNTAX_SET.find_syntax_by_token(lang)?,
            &theme,
        )
        .ok()?,
    )
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

    match highlight_code(cx.props().theme, &source, &k) {
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
    Err(HtmlError::UnAvailable(
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
    stream: &'c mut I,
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

/// Returns true if `raw_html`:
/// - starts with '<'
/// - ends with '>'
/// - does not have any '<' or '>' in between.
///
/// TODO:
/// An string attribute can a ">" character.
fn can_be_custom_component(raw_html: &str) -> bool {
    let chars: Vec<_> = raw_html.trim().chars().collect();
    let len = chars.len();
    if len < 3 {
        return false;
    };
    let (fst, middle, last) = (chars[0], &chars[1..len - 1], chars[len - 1]);
    fst == '<' && last == '>' && middle.into_iter().all(|c| c != &'<' && c != &'>')
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
        let (item, range): (Event<'a>, Range<usize>) = self.stream.next()?;
        let range = range.clone();

        let cx = self.cx;

        let rendered = match item {
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
            InlineHtml(s) => self.html(&s, range),
            Html(raw_html) => self.html(&raw_html, range),
            FootnoteReference(_) => Err(HtmlError::not_implemented("footnotes refs")),
            SoftBreak => Ok(cx.el_text(" ".into())),
            HardBreak => Ok(self.cx.el_br()),
            Rule => Ok(cx.render_rule(range)),
            TaskListMarker(m) => Ok(cx.render_tasklist_marker(m, range)),
            InlineMath(content) => render_maths(self.cx, &content, MathMode::Inline, range),
            DisplayMath(content) => render_maths(self.cx, &content, MathMode::Display, range),
        };

        Some(rendered.unwrap_or_else(|e| {
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
    pub fn new(cx: F, events: &'c mut I) -> Self {
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

    /// Try to render `raw_html` as a custom component.
    /// - If it looks like `<Component/>` and Component is registered,
    ///     render the corresponding component.
    /// - If it looks like `<Component>`, and Component is registered,
    ///     extract markdown `<Component/>` is found.
    /// In any other cases, render the string as raw html.
    ///
    /// TODO: document (and fix?) how this behaves if given an open tag and not a closing one.
    fn html(&mut self, raw_html: &str, range: Range<usize>) -> Result<F::View, HtmlError> {
        // TODO: refactor

        match &self.current_component {
            Some(current_name) => {
                if self.end_tag.is_some() {
                    return Err(HtmlError::component(
                        raw_html,
                        "please make sure there is a newline before the end of your component",
                    ));
                }
                match CustomHtmlTag::from_str(raw_html, range.start) {
                    Ok(CustomHtmlTag::End(name)) if &name == current_name => {
                        Ok(self.next().unwrap_or(self.cx.el_empty()))
                    }
                    Ok(_) => Err(HtmlError::component(
                        current_name,
                        "expected end of component",
                    )),
                    Err(e) => Err(HtmlError::syntax(e.message)),
                }
            }
            None => {
                // If making a new html tag, check if it has a name that is a valid custom component name.
                // If so, render it accordingly (as the component or error).
                // Otherwise fall through to the catch all inline html case below.
                if can_be_custom_component(raw_html) {
                    match CustomHtmlTag::from_str(raw_html, range.start) {
                        Ok(CustomHtmlTag::Inline(s)) => {
                            if self.cx.has_custom_component(&s.name) {
                                return self.custom_component_inline(s);
                            }
                        }
                        Ok(CustomHtmlTag::End(name)) => {
                            if self.cx.has_custom_component(&name) {
                                return Err(HtmlError::component(name, "expected start, not end"));
                            }
                        }
                        Ok(CustomHtmlTag::Start(s)) => {
                            if self.cx.has_custom_component(&s.name) {
                                return self.custom_component(s);
                            }
                        }
                        Err(CustomHtmlTagError {
                            name: Some(name),
                            message,
                        }) => {
                            if self.cx.has_custom_component(&name) {
                                return Err(HtmlError::component(
                                    name,
                                    format!("not a valid component: {message}"),
                                ));
                            }
                        }
                        // Component did not parse as a custom component far enough to get a name, so fall through to raw html.
                        Err(CustomHtmlTagError {
                            name: None,
                            message: _,
                        }) => {}
                    };
                }
                // Not a custom component, so render html as is without and parsing/validation.
                Ok(self
                    .cx
                    .el_span_with_inner_html(raw_html.to_string(), Default::default()))
            }
        }
    }

    /// Convert attributes from [ComponentCall] format to [MdComponentProps] format.
    fn convert_attributes(input: ComponentCall) -> BTreeMap<String, MdComponentAttribute> {
        // TODO: this should probably unescape the attribute values.
        BTreeMap::from_iter(input.attributes.iter().map(|(k, v)| {
            (
                k.to_string(),
                MdComponentAttribute {
                    value: v.to_string(),
                    range: Self::get_range(input.full_string, v, input.range_offset),
                },
            )
        }))
    }

    /// Gives the byte of `parent` that is made up of `inner`.
    fn get_range(parent: &str, inner: &str, base: usize) -> Range<usize> {
        let a = parent.as_ptr() as usize;
        let b = inner.as_ptr() as usize;

        let offset = b - a;
        assert!(offset + inner.len() <= parent.len());

        return (base + offset)..(offset + base + inner.len());
    }

    /// Renders a custom component with children.
    fn custom_component(&mut self, description: ComponentCall) -> Result<F::View, HtmlError> {
        let name: &str = &description.name;
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
        let name: &str = &description.name;
        if !self.cx.has_custom_component(name) {
            return Err(HtmlError::component(name, "not a valid component"));
        }

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
