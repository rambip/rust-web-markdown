use core::ops::Range;

use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;

use pulldown_cmark_wikilink::{Event, Tag, TagEnd, CodeBlockKind, Alignment, MathMode};

use katex;


lazy_static::lazy_static!{
    static ref SYNTAX_SET: SyntaxSet = {
        SyntaxSet::load_defaults_newlines()
    };
    static ref THEME_SET: ThemeSet = {
        ThemeSet::load_defaults()
    };
}

use crate::utils::as_closing_tag;
use super::{
    Context,
    LinkDescription,
    MdComponentProps,
    ElementAttributes
};

use super::HtmlElement::*;

use crate::component::ComponentCall;


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


fn render_code_block<'a, 'callback, F: Context<'callback>>(
    cx: &'a F,
    string_content: Option<String>,
    k: &CodeBlockKind,
    range: Range<usize>
    ) -> F::View {
    let content = match string_content {
        Some(x) => x,
        None => return cx.el(Code, cx.el_empty())
    };

    let callback = cx.make_md_handler(range.clone());

    let code_attributes = ElementAttributes{
        on_click: Some(cx.make_md_handler(range)),
        ..Default::default()
    };

    let pre_attributes = |c| ElementAttributes{
        inner_html: Some(c),
        ..Default::default()
    };

    match highlight_code(cx.props().theme, &content, &k) {
        None => cx.el_with_attributes(
            Code,
            cx.el_with_attributes(Pre, cx.el_empty(), pre_attributes(content)),
            code_attributes
        ),
        Some(x) => cx.el_with_attributes(
            Span,
            cx.el_empty(),
            ElementAttributes{
                on_click:Some(callback),
                inner_html:Some(x),
                ..Default::default()
            }
        )
    }
}

/// `render_maths(content)` returns a html node
/// with the latex content `content` compiled inside
fn render_maths<'a, 'callback, F: Context<'callback>>(cx: &'a F, content: &str, display_mode: &MathMode, range: Range<usize>) 
    -> Result<F::View, HtmlError>{
    let opts = katex::Opts::builder()
        .display_mode(*display_mode == MathMode::Display)
        .build()
        .unwrap();

    let class_name = match display_mode {
        MathMode::Inline => "math-inline",
        MathMode::Display => "math-flow",
    };

    let callback = cx.make_md_handler(range);

    match katex::render_with_opts(content, opts){
        Ok(x) => Ok(cx.el_with_attributes(
                Span,
                cx.el_empty(),
                ElementAttributes{
                    classes: vec![class_name.to_string()],
                    on_click: Some(callback),
                    inner_html: Some(x),
                    ..Default::default()
                })),
        Err(_) => HtmlError::err("invalid math")
    }
}


/// `align_string(align)` gives the css string
/// that is used to align text according to `align`
fn align_string(align: Alignment) -> String {
    match align {
        Alignment::Left => "text-align: left",
        Alignment::Right => "text-align: right",
        Alignment::Center => "text-align: center",
        Alignment::None => "",
    }.to_string()
}




pub struct HtmlError(String);

impl HtmlError {
    fn err<T>(message: &str) -> Result<T, Self>{
        Err(HtmlError(message.to_string()))
    }
}

impl ToString for HtmlError {
    fn to_string(&self) -> String {
        self.0.to_owned()
    }
}





pub struct Renderer<'a, 'c, I, F>
where I: Iterator<Item=(Event<'a>, Range<usize>)>,
      F: Context<'a>,
{
    cx: &'a F,
    stream: &'c mut I,
    // TODO: Vec<Alignment> to &[Alignment] to avoid cloning.
    // But it requires to provide the right lifetime
    column_alignment: Option<Vec<Alignment>>,
    cell_index: usize,
    end_tag: Option<TagEnd>,
    current_component: Option<String>
}


fn is_probably_custom_component(raw_html: &str) -> bool {
    raw_html.chars()
        .filter(|x| x==&'<' || x==&'>')
        .count()
        == 2
}

impl<'a, 'c, I, F> Iterator for Renderer<'a, 'c, I, F> 
where I: Iterator<Item=(Event<'a>, Range<usize>)>,
      F: Context<'a>,
{
    type Item = F::View;

    fn next(&mut self) -> Option<Self::Item> {
        use Event::*;
        let (item, range) = self.stream.next()? ;
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
            Text(s) => Ok(cx.render_text(&s, range)),
            Code(s) => Ok(cx.render_code(&s, range)),
            InlineHtml(s) => self.html(&s, range)?, // FIXME: custom component logic ?
            Html(s) => self.html(&s, range)?,
            FootnoteReference(_) => HtmlError::err("do not support footnote refs yet"),
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
                        self.cx.el_text(&e.0),
                        self.cx.el_br(),
                    ]),
                    ElementAttributes {
                        classes: vec!["error".to_string()],
                        inner_html: None,
                        on_click: None,
                        ..Default::default()
                    }
                )
            )
        )
    }
}


impl<'a, 'c, I, F> Renderer<'a, 'c, I, F> 
where I: Iterator<Item=(Event<'a>, Range<usize>)>,
      F: Context<'a>,
{
    pub fn new(cx: &'a F, events: &'c mut I)-> Self 
    {

        Self {
            cx,
            stream: events,
            column_alignment: None,
            cell_index: 0,
            end_tag: None,
            current_component: None,
        }
    }

    fn html(&mut self, s: &str, range: Range<usize>) 
        -> Option<Result<F::View, HtmlError>> {
            match (&self.current_component, self.end_tag) {
                (None, None) if is_probably_custom_component(s) => {
                    Some(self.custom_component(s))
                },
                (None, _) => {
                    let callback = self.cx.make_md_handler(range);
                    Some(Ok(self.cx.el_with_attributes(
                                Span,
                                self.cx.el_empty(),
                                ElementAttributes{
                                    inner_html: Some(s.to_string()),
                                    on_click: Some(callback),
                                    ..Default::default()
                                }
                                )
                           )
                        )
                },
                (Some(x), None) if s.trim()==format!("</{x}>") => {
                    // legit end of custom component
                    return None
                },
                (Some(x), None) if is_probably_custom_component(s) => {
                    Some(HtmlError::err(&format!("the component `{x}` is not properly closed")))
                },
                (Some(x), Some(_)) if s.trim()==format!("</{x}>") => {
                    Some(HtmlError::err(&format!("please make sure there is a newline before the end of your component")))
                },
                _ => {
                    // tries to render html as raw html anyway
                    Some(Ok(self.cx.el_with_attributes(
                                Span,
                                self.cx.el_empty(),
                                ElementAttributes{
                                    inner_html: Some(s.to_string()),
                                    ..Default::default()
                                }
                                )
                           )
                        )
                },
            }
        }

    fn custom_component(&mut self, raw_html: &str) -> Result<F::View, HtmlError> {
        let description: ComponentCall = raw_html.parse().map_err(|x| HtmlError(x))?;
        let name: &str = &description.name;
        let comp = self.cx.props().components.get(name)
            .ok_or(HtmlError(format!("{} is not a valid component", description.name)))?;

        if description.children {
            let sub_renderer = Renderer {
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
        else {
            Ok(
                F::call_html_callback(comp, MdComponentProps{
                    attributes: description.attributes, 
                    children: self.cx.el_empty()
                })
            )
        }
    }

    fn children(&mut self, tag: Tag<'a>) -> F::View {
        let sub_renderer = Renderer {
            cx: self.cx,
            stream: self.stream,
            column_alignment: self.column_alignment.clone(),
            cell_index: 0,
            end_tag: Some(as_closing_tag(&tag)),
            current_component: self.current_component.clone(),
        };
        self.cx.el_fragment(sub_renderer.collect())
    }

    fn children_text(&mut self, tag: Tag<'a>) -> Option<String> {
        let text = match self.stream.next() {
            Some((Event::Text(s), _)) => Some(s.to_string()),
            None => None,
            _ => panic!("expected string event, got something else")
        };

        let end_tag = &self.stream.next().expect("this event should be the closing tag").0;
        assert!(end_tag == &Event::End(as_closing_tag(&tag)));

        text
    }

    fn children_html(&mut self, tag: Tag<'a>) -> Option<String> {
        let text = match self.stream.next() {
            Some((Event::Html(s), _)) => Some(s.to_string()),
            None => None,
            _ => panic!("expected html event, got something else")
        };

        let end_tag = &self.stream.next().expect("this event should be the closing tag").0;
        assert!(end_tag == &Event::End(as_closing_tag(&tag)));

        text
    }

    fn render_tag(&mut self, tag: Tag<'a>, range: Range<usize>) 
    -> Result<F::View, HtmlError> 
    {
        let cx = self.cx;
        let props = self.cx.props();
        Ok(match tag.clone() {
            Tag::HtmlBlock => {
                let maybe_node = self.children_html(tag).map(
                    |c| cx.el_with_attributes(Div, cx.el_empty(),
                        ElementAttributes {
                            inner_html: Some(c),
                            ..Default::default()
                        }
                    )
                );
                cx.el_fragment(maybe_node.into_iter().collect())
            },
            Tag::Paragraph => cx.el(Paragraph, self.children(tag)),
            Tag::Heading{level, ..} => cx.el(Heading(level as u8), self.children(tag)),
            Tag::BlockQuote => cx.el(BlockQuote, self.children(tag)),
            Tag::CodeBlock(k) => 
                render_code_block(cx, self.children_text(tag), &k, range),
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
                          style:Some(align_string(align)),
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
            Tag::FootnoteDefinition(_) => return HtmlError::err("footnote not implemented"),
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

