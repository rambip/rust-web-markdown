use pulldown_cmark_wikilink::{ParserOffsetIter, LinkType, Event};
pub use pulldown_cmark_wikilink::Options;
use web_sys::MouseEvent;

use core::ops::Range;
use std::collections::HashMap;

mod render;
use render::Renderer;

mod utils;

mod component;


pub struct ElementAttributes<'callback, F: WebFramework<'callback>> {
    pub classes: Vec<String>,
    pub style: Option<String>,
    pub inner_html: Option<String>,
    pub on_click: Option<F::Callback<MouseEvent, ()>>
}

impl<'callback, F: WebFramework<'callback>> Default for ElementAttributes<'callback,  F> {
    fn default() -> Self {
        Self {
            style: None,
            classes: vec![],
            inner_html: None,
            on_click: None
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
    Code
}

pub trait WebFramework<'callback>: Clone {
    type View;
    type HtmlCallback<T: 'callback>: Clone + 'callback;
    type Callback<A: 'callback, B: 'callback>: Clone + 'callback;
    type Setter<T: 'static>: Clone;
    fn set<T>(&self, setter: &Self::Setter<T>, value: T);
    fn send_debug_info(&self, info: Vec<String>);
    fn el_with_attributes(&self, e: HtmlElement, inside: Self::View, attributes: ElementAttributes<'callback, Self>) -> Self::View;
    fn el(&self, e: HtmlElement, inside: Self::View) -> Self::View {
        self.el_with_attributes(e, inside, Default::default())
    }
    fn el_hr(&self, attributes: ElementAttributes<'callback, Self>) -> Self::View;
    fn el_br(&self)-> Self::View;
    fn el_fragment(&self, children: Vec<Self::View>) -> Self::View;
    fn el_a(&self, children: Self::View, href: &str) -> Self::View;
    fn el_img(&self, src: &str, alt: &str) -> Self::View;
    fn el_empty(&self) -> Self::View {
        self.el_fragment(vec![])
    }
    fn el_text(&self, text: &str) -> Self::View;
    fn mount_dynamic_link(&self, rel: &str, href: &str, integrity: &str, crossorigin: &str);
    fn el_input_checkbox(&self, checked: bool, attributes: ElementAttributes<'callback, Self>) -> Self::View;
    fn call_callback<A, B>(callback: &Self::Callback<A,B>, input: A) -> B;
    fn call_html_callback<T>(callback: &Self::HtmlCallback<T>, input: T) -> Self::View;
    fn make_callback<A, B, F: Fn(A)->B + 'callback>(f: F) -> Self::Callback<A, B>;
}

#[derive(Clone, Debug)]
pub struct MarkdownMouseEvent {
    /// the original mouse event triggered when a text element was clicked on
    pub mouse_event: MouseEvent,

    /// the corresponding range in the markdown source, as a slice of [`u8`][u8]
    pub position: Range<usize>,

    // TODO: add a clonable tag for the type of the element
    // pub tag: pulldown_cmark::Tag<'a>,
}


/// the description of a link, used to render it with a custom callback.
/// See [pulldown_cmark::Tag::Link] for documentation
pub struct LinkDescription<'callback, F: WebFramework<'callback>> {
    /// the url of the link
    pub url: String,

    /// the html view of the element under the link
    pub content: F::View,

    /// the title of the link. 
    /// If you don't know what it is, don't worry: it is ofter empty
    pub title: String,

    /// the type of link
    pub link_type: LinkType,

    /// wether the link is an image
    pub image: bool,
}


#[derive(PartialEq)]
pub struct MdComponentProps<'callback, F: WebFramework<'callback>> {
    pub attributes: Vec<(String, String)>,
    pub children: F::View
}


#[derive(Clone)]
pub struct MarkdownProps<'a, 'callback, F: WebFramework<'callback> + 'callback> 
{
    pub on_click: Option<&'a F::Callback<MarkdownMouseEvent, ()>>,

    pub render_links: Option<&'a F::HtmlCallback<LinkDescription<'callback, F>>>,

    pub theme: Option<&'a str>,

    pub wikilinks: bool,

    pub hard_line_breaks: bool,

    pub parse_options: Option<&'a pulldown_cmark_wikilink::Options>,

    pub components: &'a HashMap<String, F::HtmlCallback<MdComponentProps<'callback, F>>>,

    pub frontmatter: Option<&'a F::Setter<String>>
}

impl<'a, 'callback, F: WebFramework<'callback>> Copy for MarkdownProps<'a, 'callback, F> {}

pub fn render_markdown<'a, 'callback, F: WebFramework<'callback>>(
    cx: F, 
    source: &'a str, 
    props: MarkdownProps<'a, 'callback, F>
    ) -> F::View {

    let parse_options_default = Options::all();
    let options = props.parse_options.unwrap_or(&parse_options_default);
    let mut stream: Vec<_>
        = ParserOffsetIter::new_ext(source, *options, props.wikilinks).collect();

    if props.hard_line_breaks {
        for (r, _) in &mut stream {
            if *r == Event::SoftBreak {
                *r = Event::HardBreak
            }
        }
    }

    let elements = Renderer::new(cx.clone(), props, &mut stream.into_iter())
        .collect::<Vec<_>>();


    cx.mount_dynamic_link(
        "stylesheet",
        "https://cdn.jsdelivr.net/npm/katex@0.16.7/dist/katex.min.css",
        "sha384-3UiQGuEI4TTMaFmGIZumfRPtfKQ3trwQE2JgosJxCnGmQpL/lJdjpcHkaaFwHlcI",
        "anonymous"
    );

    cx.el_fragment(elements)
}
