use web_framework_markdown::{markdown_component, CowStr, MarkdownProps};

use std::collections::BTreeMap;

pub type MdComponentProps = web_framework_markdown::MdComponentProps<Element>;

use core::ops::Range;

pub use web_framework_markdown::{
    ComponentCreationError, Context, ElementAttributes, HtmlElement, LinkDescription, Options,
};

use dioxus::prelude::*;

mod substring;

pub use substring::ReadWriteBox;

pub type HtmlCallback<T> = Callback<T, Element>;

#[cfg(feature = "debug")]
pub mod debug {
    use dioxus::signals::{GlobalMemo, GlobalSignal, Signal};

    pub(crate) static DEBUG_INFO_SOURCE: GlobalSignal<Vec<String>> = Signal::global(|| Vec::new());
    pub static DEBUG_INFO: GlobalMemo<Vec<String>> = Signal::global_memo(|| DEBUG_INFO_SOURCE());
}

#[derive(Clone, PartialEq, Default, Props)]
pub struct MdProps {
    src: ReadSignal<String>,

    /// The callback called when a component is clicked.
    /// If you want to control what happens when a link is clicked,
    /// use [`render_links`][render_links]
    on_click: Option<EventHandler<MarkdownMouseEvent>>,

    ///
    render_links: Option<HtmlCallback<LinkDescription<Element>>>,

    /// the name of the theme used for syntax highlighting.
    /// Only the default themes of [syntect::Theme] are supported
    theme: Option<&'static str>,

    /// wether to enable wikilinks support.
    /// Wikilinks look like [[shortcut link]] or [[url|name]]
    #[props(default = false)]
    wikilinks: bool,

    /// wether to convert soft breaks to hard breaks.
    #[props(default = false)]
    hard_line_breaks: bool,

    /// pulldown_cmark options.
    /// See [`Options`][pulldown_cmark_wikilink::Options] for reference.
    parse_options: Option<Options>,

    #[props(default)]
    components: ReadSignal<CustomComponents>,

    frontmatter: Option<Signal<String>>,

    /// wether to preserve arbitrary html.
    /// If true, content may inject unsafe html, which could be a security or privacy risk if the input comes from an untrusted source.
    /// TODO: supporting a sanitized subset of html might be a better approach in the future.
    #[props(default = true)]
    preserve_html: bool,
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

#[derive(Clone, Copy)]
pub struct MdContext(ReadSignal<MdProps>);

/// component store.
/// It is called when therer is a `<CustomComponent>` inside the markdown source.
/// It is basically a hashmap but more efficient for a small number of items
#[derive(Default)]
pub struct CustomComponents(
    BTreeMap<String, Callback<MdComponentProps, Result<Element, ComponentCreationError>>>,
);

impl CustomComponents {
    pub fn new() -> Self {
        Self(Default::default())
    }

    /// register a new component.
    /// The function `component` takes a context and props of type `MdComponentProps`
    /// and returns html
    pub fn register<F>(&mut self, name: &'static str, component: F)
    where
        F: Fn(MdComponentProps) -> Result<Element, ComponentCreationError> + 'static,
    {
        self.0.insert(name.to_string(), Callback::new(component));
    }

    pub fn get_callback(
        &self,
        name: &str,
    ) -> Option<&Callback<MdComponentProps, Result<Element, ComponentCreationError>>> {
        self.0.get(name)
    }
}

impl<'src> Context<'src, 'static> for MdContext {
    type View = Element;

    type Handler<T: 'static> = EventHandler<T>;

    type MouseEvent = MouseEvent;

    #[cfg(feature = "debug")]
    fn send_debug_info(self, info: Vec<String>) {
        *debug::DEBUG_INFO_SOURCE.write() = info;
    }

    fn el_with_attributes(
        self,
        e: HtmlElement,
        inside: Self::View,
        attributes: ElementAttributes<EventHandler<MouseEvent>>,
    ) -> Self::View {
        let class = attributes.classes.join(" ");
        let style = attributes.style.unwrap_or_default();
        let onclick = attributes.on_click.unwrap_or_default();
        let onclick = move |e| onclick.call(e);

        match e {
            HtmlElement::Div => {
                rsx! {
                    div { onclick, style: "{style}", class: "{class}", {inside} }
                }
            }
            HtmlElement::Span => {
                rsx! {
                    span { onclick, style: "{style}", class: "{class}", {inside} }
                }
            }
            HtmlElement::Paragraph => {
                rsx! {
                    p { onclick, style: "{style}", class: "{class}", {inside} }
                }
            }
            HtmlElement::BlockQuote => {
                rsx! {
                    blockquote { onclick, style: "{style}", class: "{class}", {inside} }
                }
            }
            HtmlElement::Ul => {
                rsx! {
                    ul { onclick, style: "{style}", class: "{class}", {inside} }
                }
            }
            HtmlElement::Ol(x) => {
                rsx! {
                    ol {
                        onclick,
                        style: "{style}",
                        class: "{class}",
                        start: x as i64,
                        {inside}
                    }
                }
            }
            HtmlElement::Li => {
                rsx! {
                    li { onclick, style: "{style}", class: "{class}", {inside} }
                }
            }
            HtmlElement::Heading(1) => {
                rsx! {
                    h1 { onclick, style: "{style}", class: "{class}", {inside} }
                }
            }
            HtmlElement::Heading(2) => {
                rsx! {
                    h2 { onclick, style: "{style}", class: "{class}", {inside} }
                }
            }
            HtmlElement::Heading(3) => {
                rsx! {
                    h3 { onclick, style: "{style}", class: "{class}", {inside} }
                }
            }
            HtmlElement::Heading(4) => {
                rsx! {
                    h4 { onclick, style: "{style}", class: "{class}", {inside} }
                }
            }
            HtmlElement::Heading(5) => {
                rsx! {
                    h5 { onclick, style: "{style}", class: "{class}", {inside} }
                }
            }
            HtmlElement::Heading(6) => {
                rsx! {
                    h6 { onclick, style: "{style}", class: "{class}", {inside} }
                }
            }
            HtmlElement::Heading(_) => panic!(),
            HtmlElement::Table => {
                rsx! {
                    table { onclick, style: "{style}", class: "{class}", {inside} }
                }
            }
            HtmlElement::Thead => {
                rsx! {
                    thead { onclick, style: "{style}", class: "{class}", {inside} }
                }
            }
            HtmlElement::Trow => {
                rsx! {
                    tr { onclick, style: "{style}", class: "{class}", {inside} }
                }
            }
            HtmlElement::Tcell => {
                rsx! {
                    td { onclick, style: "{style}", class: "{class}", {inside} }
                }
            }
            HtmlElement::Italics => {
                rsx! {
                    i { onclick, style: "{style}", class: "{class}", {inside} }
                }
            }
            HtmlElement::Bold => {
                rsx! {
                    b { onclick, style: "{style}", class: "{class}", {inside} }
                }
            }
            HtmlElement::StrikeThrough => {
                rsx! {
                    s { onclick, style: "{style}", class: "{class}", {inside} }
                }
            }
            HtmlElement::Pre => {
                rsx! {
                    p { onclick, style: "{style}", class: "{class}", {inside} }
                }
            }
            HtmlElement::Code => {
                rsx! {
                    code { onclick, style: "{style}", class: "{class}", {inside} }
                }
            }
        }
    }

    fn el_span_with_inner_html(
        self,
        inner_html: String,
        attributes: ElementAttributes<EventHandler<MouseEvent>>,
    ) -> Self::View {
        let class = attributes.classes.join(" ");
        let style = attributes.style.unwrap_or_default();
        let onclick = move |e| {
            if let Some(f) = &attributes.on_click {
                f.call(e)
            }
        };
        let props = self.0();
        if props.preserve_html {
            rsx! {
                span {
                    dangerous_inner_html: "{inner_html}",
                    style: "{style}",
                    class: "{class}",
                    onclick,
                }
            }
        } else {
            rsx! {
                span { style: "{style}", class: "{class}", onclick, "{inner_html}" }
            }
        }
    }

    fn el_hr(self, attributes: ElementAttributes<EventHandler<MouseEvent>>) -> Self::View {
        let class = attributes.classes.join(" ");
        let style = attributes.style.unwrap_or_default();
        let onclick = move |e| {
            if let Some(f) = &attributes.on_click {
                f.call(e)
            }
        };
        rsx!(hr {
            onclick,
            style: "{style}",
            class: "{class}"
        })
    }

    fn el_br(self) -> Self::View {
        rsx!(br {})
    }

    fn el_fragment(self, children: Vec<Self::View>) -> Self::View {
        rsx! {
            {children.into_iter()}
        }
    }

    fn el_a(self, children: Self::View, href: String) -> Self::View {
        rsx! {
            a { href: "{href}", {children} }
        }
    }

    fn el_img(self, src: String, alt: String) -> Self::View {
        rsx!(img {
            src: "{src}",
            alt: "{alt}"
        })
    }

    fn el_text<'a>(self, text: CowStr<'a>) -> Self::View {
        rsx! {
            {text.as_ref()}
        }
    }

    fn el_input_checkbox(
        self,
        checked: bool,
        attributes: ElementAttributes<EventHandler<MouseEvent>>,
    ) -> Self::View {
        let class = attributes.classes.join(" ");
        let style = attributes.style.unwrap_or_default();
        let onclick = move |e| {
            if let Some(f) = &attributes.on_click {
                f.call(e)
            }
        };
        rsx!(input {
            r#type: "checkbox",
            checked,
            style: "{style}",
            class: "{class}",
            onclick,
        })
    }

    fn props(self) -> MarkdownProps {
        let props = self.0();

        MarkdownProps {
            hard_line_breaks: props.hard_line_breaks,
            wikilinks: props.wikilinks,
            parse_options: props.parse_options,
            theme: props.theme,
        }
    }

    fn call_handler<T: 'static>(callback: &Self::Handler<T>, input: T) {
        callback.call(input)
    }

    fn make_md_handler(
        self,
        position: std::ops::Range<usize>,
        stop_propagation: bool,
    ) -> Self::Handler<MouseEvent> {
        let on_click = self.0().on_click.as_ref().cloned();

        EventHandler::new(move |e: MouseEvent| {
            if stop_propagation {
                e.stop_propagation()
            }

            let report = MarkdownMouseEvent {
                position: position.clone(),
                mouse_event: e,
            };

            on_click.map(|x| x.call(report));
        })
    }

    fn set_frontmatter(&mut self, frontmatter: String) {
        self.0().frontmatter.as_mut().map(|x| x.set(frontmatter));
    }

    fn has_custom_links(self) -> bool {
        self.0().render_links.is_some()
    }

    fn render_links(self, link: LinkDescription<Self::View>) -> Result<Self::View, String> {
        // TODO: remove the unwrap call
        Ok(self.0().render_links.as_ref().unwrap()(link))
    }

    fn has_custom_component(self, name: &str) -> bool {
        self.0().components.read().get_callback(name).is_some()
    }

    fn render_custom_component(
        self,
        name: &str,
        input: MdComponentProps,
    ) -> Result<Self::View, ComponentCreationError> {
        let f: Callback<_, _> = self.0()
            .components
            .read()
            .get_callback(name)
            .unwrap()
            .clone();
        f(input)
    }
}

#[allow(non_snake_case)]
pub fn Markdown(props: MdProps) -> Element {
    let src: String = props.src.to_string();
    let signal: Signal<MdProps> = Signal::new(props);
    let child = markdown_component(MdContext(signal.into()), &src);
    #[cfg(feature = "maths")]
    rsx! {
        document::Style { href: web_framework_markdown::MATH_STYLE_SHEET_LINK.href }
        {child}
    }
    #[cfg(not(feature = "maths"))]
    rsx! {
        {child}
    }
}
