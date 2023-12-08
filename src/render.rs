use core::ops::Range;

use core::marker::PhantomData;

use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;

use pulldown_cmark_wikilink::{Event, Tag, TagEnd, CodeBlockKind, Alignment, MathMode};

use katex;

use crate::utils::as_closing_tag;
use super::{
    Context,
    LinkDescription,
    MdComponentProps,
    ElementAttributes
};

use super::HtmlElement::*;

use crate::component::{ComponentCall, Stuff};

// load the default syntect options to highlight code
lazy_static::lazy_static!{
    static ref SYNTAX_SET: SyntaxSet = {
        SyntaxSet::load_defaults_newlines()
    };
    static ref THEME_SET: ThemeSet = {
        ThemeSet::load_defaults()
    };
}


pub enum HtmlError {
    NotImplemented(String),
    Syntax(String),
    CustomComponent{name: String, msg: String},
    Math,
}

impl HtmlError {
    fn not_implemented(message: impl ToString) -> Self{
        HtmlError::NotImplemented(message.to_string())
    }
    fn syntax(message: impl ToString) -> Self{
        HtmlError::Syntax(message.to_string())
    }
    fn component(name: impl ToString, msg: impl ToString) -> Self {
        HtmlError::CustomComponent{
            name: name.to_string(),
            msg: msg.to_string(),
        }
    }
}

impl ToString for HtmlError {
    fn to_string(&self) -> String {
        match self {
            HtmlError::Math => "invalid math".to_string(),
            HtmlError::NotImplemented(s) => 
                format!("`{s}`: not implemented"),
            HtmlError::CustomComponent{name, msg} =>
                format!("Custom component `{name}` failed: `{msg}`"),
            HtmlError::Syntax(s) =>
                format!("syntax error: {s}")
        }
    }
}




/// `highlight_code(content, ss, ts)` render the content `content`
/// with syntax highlighting
fn highlight_code(theme_name: Option<&str>, content: &str, kind: &CodeBlockKind) -> Option<String> {
    let lang = match kind {
        CodeBlockKind::Fenced(x) => x,
        CodeBlockKind::Indented => return None
    };

    let theme_name = theme_name
        .clone()
        .unwrap_or("base16-ocean.light");
    let theme = THEME_SET.themes.get(theme_name)
        .expect("unknown theme")
        .clone();

    Some(
        syntect::html::highlighted_html_for_string(
            content,
            &SYNTAX_SET,
            SYNTAX_SET.find_syntax_by_token(lang)?,
            &theme
            ).ok()?
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
    range: Range<usize>
    ) -> F::View {

    let code_attributes = ElementAttributes{
        on_click: Some(cx.make_md_handler(range, true)),
        ..Default::default()
    };

    match highlight_code(cx.props().theme, &source, &k) {
        None => cx.el_with_attributes(
            Code,
            cx.el(Code, cx.el_text(source.into())),
            code_attributes
        ),
        Some(x) => cx.el_span_with_inner_html(x, code_attributes)
    }
}

/// `render_maths(content)` returns a html node
/// with the latex content `content` compiled inside
fn render_maths<'a, 'callback, F: Context<'a, 'callback>>(cx: F, content: &str, display_mode: &MathMode, range: Range<usize>) 
    -> Result<F::View, HtmlError>{
    let opts = katex::Opts::builder()
        .display_mode(*display_mode == MathMode::Display)
        .build()
        .unwrap();

    let class_name = match display_mode {
        MathMode::Inline => "math-inline",
        MathMode::Display => "math-flow",
    };

    let callback = cx.make_md_handler(range, true);

    let attributes = ElementAttributes{
            classes: vec![class_name.to_string()],
            on_click: Some(callback),
            ..Default::default()
    };

    match katex::render_with_opts(content, opts){
        Ok(x) => Ok(cx.el_span_with_inner_html(x, attributes)),
        Err(_) => Err(HtmlError::Math),
    }
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
where I: Iterator<Item=(Event<'a>, Range<usize>)>,
      'callback: 'a,
      F: Context<'a, 'callback>,
{
    __marker : PhantomData<&'callback ()>,
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
    current_component: Option<String>
}

/// returns true if `raw_html`:
/// - starts with '<'
/// - ends with '>'
/// - does not have any '<' or '>' in between
fn can_be_custom_component(raw_html: &str) -> bool {
    let chars: Vec<_> = raw_html.trim().chars().collect();
    let len = chars.len();
    if len==0 {return false};
    let (fst, middle, last) = (chars[0], &chars[1..len-1], chars[len-1]);
    fst == '<' && last == '>' && middle.into_iter().all(|c| c!=&'<' && c!=&'>')
}

impl<'a, 'callback, 'c, I, F> Iterator for Renderer<'a, 'callback, 'c, I, F> 
where I: Iterator<Item=(Event<'a>, Range<usize>)>,
      'callback: 'a,
      F: Context<'a, 'callback>,
{
    type Item = F::View;

    fn next(&mut self) -> Option<Self::Item> {
        use Event::*;
        let (item, range): (Event<'a>, Range<usize>) = self.stream.next()? ;
        let range = range.clone();

        let cx = self.cx;

        let rendered = match item {
            Start(t) => self.render_tag(t, range),
            End(end) => {
                // check if the closing tag is the tag that was open
                // when this renderer was created
                match self.end_tag {
                    Some(t) if t == end => return None,
                    Some(t) => panic!("{t:?} is a wrong closing tag"),
                    None => panic!("didn't expect a closing tag")
                }
            },
            Text(s) => Ok(cx.render_text(s, range)),
            Code(s) => Ok(cx.render_code(s, range)),
            InlineHtml(s) => {
                let attributes = ElementAttributes {
                    on_click: Some(self.cx.make_md_handler(range, false)),
                    ..ElementAttributes::default()
                };
                Ok(self.cx.el_span_with_inner_html(s.to_string(), attributes))
            },
            Html(_) => panic!("html outside html block"), 
            FootnoteReference(_) => Err(HtmlError::not_implemented("footnotes refs")),
            SoftBreak => Ok(self.next()?),
            HardBreak => Ok(self.cx.el_br()),
            Rule => Ok(cx.render_rule(range)),
            TaskListMarker(m) => Ok(cx.render_tasklist_marker(m, range)),
            Math(disp, content) => render_maths(self.cx, &content, &disp, range),
        };

        Some(
            rendered.unwrap_or_else(|e| self.cx.el_with_attributes(
                    Span,
                    self.cx.el_fragment(vec![
                        self.cx.el_text(e.to_string().into()),
                        self.cx.el_br(),
                    ]),
                    ElementAttributes {
                        classes: vec!["error".to_string()],
                        on_click: None,
                        ..Default::default()
                    }
                )
            )
        )
    }
}


impl<'a, 'callback, 'c, I, F> Renderer<'a, 'callback, 'c, I, F> 
where I: Iterator<Item=(Event<'a>, Range<usize>)>,
      F: Context<'a, 'callback>,
{
    /// creates a new renderer from a stream of events.
    /// It returns an iterator of [`F::View`]
    pub fn new(cx: F, events: &'c mut I)-> Self 
    {

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

    /// try to render `raw_html` as a custom component.
    /// - if it looks like `<Component/>` and Component is registered,
    ///     it will render the corresponding component
    /// - it it looks like `<Component>`, and Component is registered, 
    /// it will extract markdown until it finds `<Component/>`
    /// In any other cases, it will render the strinng as raw html.
    fn html(&mut self, raw_html: &str, _range: Range<usize>) 
        -> Result<F::View, HtmlError> {

            // TODO: refactor

            match &self.current_component {
                Some(current_name) => {
                    if self.end_tag.is_some() {
                        return Err(HtmlError::component(raw_html, 
                                                        "please make sure there is a newline before the end of your component"))
                    }
                    match raw_html.parse() {
                        Ok(Stuff::End(name)) if &name==current_name => {
                            Ok(self.next().unwrap_or(self.cx.el_empty()))
                        },
                        Ok(_) => Err(HtmlError::component(current_name, 
                                                          "expected end of component")),
                        Err(e) => Err(HtmlError::syntax(e))
                    }
                }
                None => {
                    if can_be_custom_component(raw_html) {
                        match raw_html.parse() {
                            Ok(Stuff::Inline(s)) => self.custom_component_inline(s),
                            Ok(Stuff::End(name)) => Err(
                                HtmlError::component(name, "expected start, not end")),
                            Ok(Stuff::Start(s)) => self.custom_component(s),
                            Err(e) => Err(HtmlError::syntax(e))
                        }
                    }
                    else {
                        Ok(self.cx.el_span_with_inner_html(raw_html.to_string(), 
                                                           Default::default()))
                    }
                }
            }
        }

    /// renders a custom component with childrens
    fn custom_component(&mut self, description: ComponentCall) -> Result<F::View, HtmlError> {
        let name: &str = &description.name;
        let comp = self.cx.props().components.get(name)
            .ok_or(HtmlError::component(name, "not a valid component"))?;

        let sub_renderer = Renderer {
            __marker: PhantomData,
            cx: self.cx,
            stream: self.stream,
            column_alignment: self.column_alignment.clone(),
            cell_index: 0,
            end_tag: self.end_tag,
            current_component: Some(description.name)
        };
        let children = self.cx.el_fragment(sub_renderer.collect());
        Ok(
            F::call_html_callback(comp, MdComponentProps{
            attributes: description.attributes,
            children
        }))
    }

    /// renders a custom component without childrens
    fn custom_component_inline(&mut self, description: ComponentCall) -> Result<F::View, HtmlError> {
        let name: &str = &description.name;
        let comp = self.cx.props().components.get(name)
            .ok_or(HtmlError::component(name, "not a valid component"))?;
        Ok(
            F::call_html_callback(comp, MdComponentProps{
                attributes: description.attributes, 
                children: self.cx.el_empty()
            })
          )
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
            end_tag: Some(as_closing_tag(&tag)),
            current_component: self.current_component.clone(),
        };
        self.cx.el_fragment(sub_renderer.collect())
    }

    /// extract the text from the next text event
    fn children_text(&mut self, tag: Tag<'a>) -> Option<String> {
        let text = match self.stream.next() {
            Some((Event::Text(s), _)) => Some(s.to_string()),
            None => None,
            _ => panic!("expected string event, got something else")
        };

        self.assert_closing_tag(as_closing_tag(&tag));
        text
    }

    // check that the closing tag is what was expected
    fn assert_closing_tag(&mut self, end: TagEnd) {
        let end_tag = &self.stream.next().expect("this event should be the closing tag").0;
        assert!(end_tag == &Event::End(end));
    }

    fn render_tag(&mut self, tag: Tag<'a>, range: Range<usize>) 
    -> Result<F::View, HtmlError> 
    {
        let cx = self.cx;
        let props = self.cx.props();
        Ok(match tag.clone() {
            Tag::HtmlBlock => {
                let raw_html = match self.stream.next() {
                    Some((Event::Html(s), _)) => s.to_string(),
                    None => panic!("empty html"),
                    _ => panic!("expected html event, got something else")
                };
                self.assert_closing_tag(TagEnd::HtmlBlock);
                self.html(&raw_html, range)?
            },
            Tag::Paragraph => cx.el(Paragraph, self.children(tag)),
            Tag::Heading{level, ..} => cx.el(Heading(level as u8), self.children(tag)),
            Tag::BlockQuote => cx.el(BlockQuote, self.children(tag)),
            Tag::CodeBlock(k) => 
                render_code_block(cx, self.children_text(tag).unwrap_or_default(), &k, range),
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
                cx.el_with_attributes(Tcell, self.children(tag), 
                      ElementAttributes{
                          style:Some(align_string(align).to_string()),
                          ..Default::default()}
                )
            },
            Tag::Emphasis => cx.el(Italics, self.children(tag)),
            Tag::Strong => cx.el(Bold, self.children(tag)),
            Tag::Strikethrough => cx.el(StrikeThrough, self.children(tag)),
            Tag::Image{link_type, dest_url, title, ..} => {
                let description = LinkDescription {
                    url: dest_url.to_string(),
                    title: title.to_string(),
                    content: self.children(tag),
                    link_type,
                    image: true,
                };
                cx.render_link(description)
            },
            Tag::Link{link_type, dest_url, title, ..} => {
                let description = LinkDescription {
                    url: dest_url.to_string(),
                    title: title.to_string(),
                    content: self.children(tag),
                    link_type,
                    image: false,
                };
                cx.render_link(description)
            },
            Tag::FootnoteDefinition(_) => 
                return Err(HtmlError::not_implemented("footnote not implemented")),
            Tag::MetadataBlock{..} => {
                let c = self.children_text(tag);
                match (&props.frontmatter, c){
                    (Some(setter), Some(text)) => cx.set(setter, text),
                    _ => ()
                };
                cx.el_empty()
            }
        }
        )
    }
}

