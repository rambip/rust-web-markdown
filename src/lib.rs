use pulldown_cmark_wikilink::{ParserOffsetIter, LinkType, Event};
pub use pulldown_cmark_wikilink::{Options, CowStr};

use core::ops::Range;
use std::collections::BTreeMap;

mod render;
use render::Renderer;

mod utils;

mod component;


pub struct ElementAttributes<H> {
    pub classes: Vec<String>,
    pub style: Option<String>,
    pub on_click: Option<H>
}

impl<H> Default for ElementAttributes<H> {
    fn default() -> Self {
        Self {
            style: None,
            classes: vec![],
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

pub trait Context<'a, 'callback>: Copy + 'a
where 'callback: 'a
{
    type View: Clone + 'callback;
    type HtmlCallback<T: 'callback>: Clone + 'callback;
    type Handler<T: 'callback>: 'callback;
    type MouseEvent: 'static;
    /// get all the properties from the context
    fn props(self) -> MarkdownProps<'a, 'callback, Self>;

    /// write the frontmatter (or metadata) string 
    /// present at the top of the markdown source
    fn set_frontmatter(self, frontmatter: String);

    fn render_links(self, link: &LinkDescription<Self::View>) 
        -> Option<Result<Self::View, String>>;

    /// calls a callback with the given input
    fn call_handler<T>(callback: &Self::Handler<T>, input: T);

    /// creates a callback that will fire when the user clicks on markdown
    fn make_md_handler(self, position: Range<usize>, stop_propagation: bool) -> Self::Handler<Self::MouseEvent>;

    #[cfg(feature="debug")]
    fn send_debug_info(self, info: Vec<String>);


    /// creates a html element
    /// `attributes` contains the html attributes for this element
    fn el_with_attributes(self, e: HtmlElement, inside: Self::View, attributes: ElementAttributes<Self::Handler<Self::MouseEvent>>) -> Self::View;

    /// creates a html element, with default attributes
    fn el(self, e: HtmlElement, inside: Self::View) -> Self::View {
        self.el_with_attributes(e, inside, Default::default())
    }

    /// renders raw html, inside a span
    fn el_span_with_inner_html(self, inner_html: String, attributes: ElementAttributes<Self::Handler<Self::MouseEvent>>) -> Self::View;

    /// renders a `hr` element, with attributes
    fn el_hr(self, attributes: ElementAttributes<Self::Handler<Self::MouseEvent>>) -> Self::View;

    /// renders a `br` element
    fn el_br(self)-> Self::View;

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
    fn el_input_checkbox(self, checked: bool, attributes: ElementAttributes<Self::Handler<Self::MouseEvent>>) -> Self::View;


    /// add a styleshit to the markdown component
    fn mount_dynamic_link(self, rel: &str, href: &str, integrity: &str, crossorigin: &str);

    fn render_tasklist_marker(self, m: bool, position: Range<usize>) -> Self::View {
        let attributes = ElementAttributes {
            on_click: Some(self.make_md_handler(position, true)),
            ..Default::default()
        };
        self.el_input_checkbox(m, attributes)
    }

    fn render_rule(self, range: Range<usize>) -> Self::View {
        let attributes = ElementAttributes{
            on_click: Some(self.make_md_handler(range, false)),
            ..Default::default()
        };
        self.el_hr(attributes)
    }


    fn render_code(self, s: CowStr<'a>, range: Range<usize>) -> Self::View {
        let callback = self.make_md_handler(range.clone(), false);
        let attributes = ElementAttributes{
            on_click: Some(callback),
            ..Default::default()
        };
        self.el_with_attributes(HtmlElement::Code, self.el_text(s), attributes)
    }


    fn render_text(self, s: CowStr<'a>, range: Range<usize>) -> Self::View{
        let callback = self.make_md_handler(range, false);
        let attributes = ElementAttributes{
            on_click: Some(callback),
            ..Default::default()
        };
        self.el_with_attributes(HtmlElement::Span, self.el_text(s), attributes)
    }


    fn render_link(self, link: LinkDescription<Self::View>) 
        -> Result<Self::View, String>
    {
        match (self.render_links(&link), link.image) {
            (Some(v), _) => v,
            (None, false) => Ok(self.el_a(link.content, link.url)),
            (None, true) => Ok(self.el_img(link.url, link.title)),
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


pub enum HtmlError {
    NotImplemented(String),
    Link(String),
    Syntax(String),
    CustomComponent{name: String, msg: String},
    Math,
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
/// ```rust
/// MdComponentProps {
///     attributes: BTreeMap::from([("color", "blue"), ("size", "5")]),
///     children: ... // html view of **hey**
/// }
/// ```
pub struct MdComponentProps<V> {
    pub attributes: BTreeMap<String, String>,
    pub children: V
}

impl<V> MdComponentProps<V> {
    pub fn get(&self, name: &str) 
        -> Result<String, String> {
        match self.attributes.get(name) {
            Some(x) => Ok(x.clone()),
            None => Err("unknown attribute".to_string())
        }
    }
}


pub struct CustomComponents<'a, 'callback, F: Context<'a, 'callback>>
    (BTreeMap<&'static str, Box<dyn Fn(F, MdComponentProps<F::View>) -> Result<F::View, HtmlError>>>);

impl<'a, 'callback, F> CustomComponents<'a, 'callback, F> 
where F: Context<'a, 'callback>
{
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn register<ERR>(&mut self, name: &'static str, component: impl Fn(F, MdComponentProps<F::View>) -> Result<F::View, ERR> + 'static) 
        where ERR: std::fmt::Debug
    {
        self.0.insert(name, Box::new(move |cx, props| {
            component(cx, props).map_err(|err| HtmlError::CustomComponent {
                name: name.to_string(),
                msg: format!("{err:?}"),
            })
        }));
    }

    fn get(&self, name: &str) -> Option<&Box<dyn Fn(F, MdComponentProps<F::View>) -> Result<F::View, HtmlError>>> {
        self.0.get(name)
    }
}
    

pub struct MarkdownProps<'a, 'callback, F: Context<'a, 'callback>>
{
    pub hard_line_breaks: bool,

    pub wikilinks: bool,

    pub parse_options: Option<&'a pulldown_cmark_wikilink::Options>,

    pub components: &'a CustomComponents<'a, 'callback, F>,

    pub theme: Option<&'a str>,
}

pub fn render_markdown<'a, 'callback, F: Context<'a, 'callback>>(
    cx: F, 
    source: &'a str, 
    ) -> F::View 
{

    let parse_options_default = Options::all();
    let options = cx.props().parse_options.unwrap_or(&parse_options_default);
    let mut stream: Vec<_>
        = ParserOffsetIter::new_ext(source, *options, cx.props().wikilinks).collect();

    #[cfg(feature="debug")]
    {
        let debug_info : Vec<String> = stream.iter().map(|x| format!("{:?}", x)).collect();
        cx.send_debug_info(debug_info)
    }

    if cx.props().hard_line_breaks {
        for (r, _) in &mut stream {
            if *r == Event::SoftBreak {
                *r = Event::HardBreak
            }
        }
    }

    let elements = Renderer::new(cx, &mut stream.into_iter())
        .collect::<Vec<_>>();


    cx.mount_dynamic_link(
        "stylesheet",
        "https://cdn.jsdelivr.net/npm/katex@0.16.7/dist/katex.min.css",
        "sha384-3UiQGuEI4TTMaFmGIZumfRPtfKQ3trwQE2JgosJxCnGmQpL/lJdjpcHkaaFwHlcI",
        "anonymous"
    );

    cx.el_fragment(elements)
}
