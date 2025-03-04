use rust_web_markdown::{render_markdown, CowStr};

use std::collections::BTreeMap;

pub type MdComponentProps<'a> = rust_web_markdown::MdComponentProps<Element<'a>>;

use core::ops::Range;

pub use rust_web_markdown::{
    ComponentCreationError, Context, ElementAttributes, HtmlElement, LinkDescription, Options,
};

use dioxus::prelude::*;

use std::rc::Rc;

pub type HtmlCallback<'a, T> = Rc<dyn Fn(&'a ScopeState, T) -> Element<'a>>;

#[cfg(feature = "debug")]
pub mod debug {
    #[derive(Clone)]
    pub struct EventInfo(pub Vec<String>);
}

#[derive(Props)]
pub struct MdProps<'a> {
    src: &'a str,

    /// the callback called when a component is clicked.
    /// if you want to controll what happens when a link is clicked,
    /// use [`render_links`][render_links]
    on_click: Option<EventHandler<'a, MarkdownMouseEvent>>,

    ///
    render_links: Option<HtmlCallback<'a, LinkDescription<Element<'a>>>>,

    /// the name of the theme used for syntax highlighting.
    /// Only the default themes of [syntect::Theme] are supported
    theme: Option<String>,

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
    components: CustomComponents<'a>,

    frontmatter: Option<UseState<String>>,
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
pub struct MdContext<'a>(pub &'a Scoped<'a, MdProps<'a>>);

/// component store.
/// It is called when therer is a `<CustomComponent>` inside the markdown source.
/// It is basically a hashmap but more efficient for a small number of items
pub struct CustomComponents<'a>(
    BTreeMap<
        &'static str,
        Box<
            dyn Fn(
                &'a ScopeState,
                MdComponentProps<'a>,
            ) -> Result<Element<'a>, ComponentCreationError>,
        >,
    >,
);

impl Default for CustomComponents<'_> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<'a> CustomComponents<'a> {
    pub fn new() -> Self {
        Self(Default::default())
    }

    /// register a new component.
    /// The function `component` takes a context and props of type `MdComponentProps`
    /// and returns html
    pub fn register<F>(&mut self, name: &'static str, component: F)
    where
        F: Fn(&'a ScopeState, MdComponentProps<'a>) -> Result<Element<'a>, ComponentCreationError>
            + 'static,
    {
        self.0.insert(name, Box::new(component));
    }
}

impl<'a> Context<'a, 'a> for MdContext<'a> {
    type View = Element<'a>;

    type Handler<T: 'a> = EventHandler<'a, T>;

    type MouseEvent = MouseEvent;

    #[cfg(feature = "debug")]
    fn send_debug_info(self, info: Vec<String>) {
        let debug = use_shared_state::<debug::EventInfo>(self.0).unwrap();
        // to avoid re-rendering the parent component
        // if not needed
        if *debug.read().0 != info {
            debug.write().0 = info
        }
    }

    fn el_with_attributes(
        self,
        e: HtmlElement,
        inside: Self::View,
        attributes: ElementAttributes<EventHandler<'a, MouseEvent>>,
    ) -> Self::View {
        let class = attributes.classes.join(" ");
        let style = attributes.style.unwrap_or_default();
        let onclick = attributes.on_click.unwrap_or_default();
        let onclick = move |e| onclick.call(e);

        let vnode = match e {
            HtmlElement::Div => {
                rsx! {div {onclick:onclick, style: "{style}", class: "{class}", inside } }
            }
            HtmlElement::Span => {
                rsx! {span {onclick: onclick, style: "{style}", class: "{class}", inside } }
            }
            HtmlElement::Paragraph => {
                rsx! {p {onclick: onclick, style: "{style}", class: "{class}", inside } }
            }
            HtmlElement::BlockQuote => {
                rsx! {blockquote {onclick: onclick, style: "{style}", class: "{class}", inside } }
            }
            HtmlElement::Ul => {
                rsx! {ul {onclick: onclick, style: "{style}", class: "{class}", inside } }
            }
            HtmlElement::Ol(x) => {
                rsx! {ol {onclick: onclick, style: "{style}", class: "{class}", start: x as i64, inside } }
            }
            HtmlElement::Li => {
                rsx! {li {onclick: onclick, style: "{style}", class: "{class}", inside } }
            }
            HtmlElement::Heading(1) => {
                rsx! {h1 {onclick: onclick, style: "{style}", class: "{class}", inside } }
            }
            HtmlElement::Heading(2) => {
                rsx! {h2 {onclick: onclick, style: "{style}", class: "{class}", inside } }
            }
            HtmlElement::Heading(3) => {
                rsx! {h3 {onclick: onclick, style: "{style}", class: "{class}", inside } }
            }
            HtmlElement::Heading(4) => {
                rsx! {h4 {onclick: onclick, style: "{style}", class: "{class}", inside } }
            }
            HtmlElement::Heading(5) => {
                rsx! {h5 {onclick: onclick, style: "{style}", class: "{class}", inside } }
            }
            HtmlElement::Heading(6) => {
                rsx! {h6 {onclick: onclick, style: "{style}", class: "{class}", inside } }
            }
            HtmlElement::Heading(_) => panic!(),
            HtmlElement::Table => {
                rsx! {table {onclick: onclick, style: "{style}", class: "{class}", inside } }
            }
            HtmlElement::Thead => {
                rsx! {thead {onclick: onclick, style: "{style}", class: "{class}", inside } }
            }
            HtmlElement::Trow => {
                rsx! {tr {onclick: onclick, style: "{style}", class: "{class}", inside } }
            }
            HtmlElement::Tcell => {
                rsx! {td {onclick: onclick, style: "{style}", class: "{class}", inside } }
            }
            HtmlElement::Italics => {
                rsx! {i {onclick: onclick, style: "{style}", class: "{class}", inside } }
            }
            HtmlElement::Bold => {
                rsx! {b {onclick: onclick, style: "{style}", class: "{class}", inside } }
            }
            HtmlElement::StrikeThrough => {
                rsx! {s {onclick: onclick, style: "{style}", class: "{class}", inside } }
            }
            HtmlElement::Pre => {
                rsx! {p {onclick: onclick, style: "{style}", class: "{class}", inside } }
            }
            HtmlElement::Code => {
                rsx! {code {onclick: onclick, style: "{style}", class: "{class}", inside } }
            }
        };

        let r: Element<'a> = self.0.render(vnode);
        r
    }

    fn el_span_with_inner_html(
        self,
        inner_html: String,
        attributes: ElementAttributes<EventHandler<'a, MouseEvent>>,
    ) -> Self::View {
        let class = attributes.classes.join(" ");
        let style = attributes.style.unwrap_or_default();
        let onclick = move |e| {
            if let Some(f) = &attributes.on_click {
                f.call(e)
            }
        };
        self.0.render(rsx! {
            span {
                dangerous_inner_html: "{inner_html}",
                style: "{style}",
                class: "{class}",
                onclick: onclick
            }
        })
    }

    fn el_hr(self, attributes: ElementAttributes<EventHandler<'a, MouseEvent>>) -> Self::View {
        let class = attributes.classes.join(" ");
        let style = attributes.style.unwrap_or_default();
        let onclick = move |e| {
            if let Some(f) = &attributes.on_click {
                f.call(e)
            }
        };
        self.0.render(rsx!(hr {
            onclick: onclick,
            style: "{style}",
            class: "{class}"
        }))
    }

    fn el_br(self) -> Self::View {
        self.0.render(rsx!(br {}))
    }

    fn el_fragment(self, children: Vec<Self::View>) -> Self::View {
        self.0.render(rsx! {children.into_iter()})
    }

    fn el_a(self, children: Self::View, href: String) -> Self::View {
        self.0.render(rsx! {a {href: "{href}", children}})
    }

    fn el_img(self, src: String, alt: String) -> Self::View {
        self.0.render(rsx!(img {
            src: "{src}",
            alt: "{alt}"
        }))
    }

    fn el_text(self, text: CowStr<'a>) -> Self::View {
        self.0.render(rsx! {text.as_ref()})
    }

    fn mount_dynamic_link(self, _rel: &str, _href: &str, _integrity: &str, _crossorigin: &str) {
        // let create_eval = use_eval(self.0);

        // let eval = create_eval(
        //     r#"
        //     // https://stackoverflow.com/a/18510577
        //     let rel = await dioxus.recv();
        //     let href = await dioxus.recv();
        //     let integrity = await dioxus.recv();
        //     let crossorigin = await dioxus.recv();
        //     var newstyle = document.createElement("link"); // Create a new link Tag

        //     newstyle.setAttribute("rel", rel);
        //     newstyle.setAttribute("type", "text/css");
        //     newstyle.setAttribute("href", href);
        //     newstyle.setAttribute("crossorigin", crossorigin);
        //     newstyle.setAttribute("integrity", integrity);
        //     document.getElementsByTagName("head")[0].appendChild(newstyle);
        //     "#,
        // )
        // .unwrap();

        // // You can send messages to JavaScript with the send method
        // eval.send(rel.into()).unwrap();
        // eval.send(href.into()).unwrap();
        // eval.send(integrity.into()).unwrap();
        // eval.send(crossorigin.into()).unwrap();
    }

    fn el_input_checkbox(
        self,
        checked: bool,
        attributes: ElementAttributes<EventHandler<'a, MouseEvent>>,
    ) -> Self::View {
        let class = attributes.classes.join(" ");
        let style = attributes.style.unwrap_or_default();
        let onclick = move |e| {
            if let Some(f) = &attributes.on_click {
                f.call(e)
            }
        };
        self.0.render(rsx!(input {
            r#type: "checkbox",
            checked: checked,
            style: "{style}",
            class: "{class}",
            onclick: onclick
        }))
    }

    fn props(self) -> rust_web_markdown::MarkdownProps<'a> {
        let props = self.0.props;

        rust_web_markdown::MarkdownProps {
            hard_line_breaks: props.hard_line_breaks,
            wikilinks: props.wikilinks,
            parse_options: props.parse_options.as_ref(),
            theme: props.theme.as_deref(),
        }
    }

    fn call_handler<T: 'a>(callback: &Self::Handler<T>, input: T) {
        callback.call(input)
    }

    fn make_md_handler(
        self,
        position: std::ops::Range<usize>,
        stop_propagation: bool,
    ) -> Self::Handler<MouseEvent> {
        let on_click = self.0.props.on_click.as_ref();

        self.0.event_handler(move |e: MouseEvent| {
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

    fn set_frontmatter(self, frontmatter: String) {
        self.0
            .props
            .frontmatter
            .as_ref()
            .map(|x| x.set(frontmatter));
    }

    fn has_custom_links(self) -> bool {
        self.0.props.render_links.is_some()
    }

    fn render_links(self, link: LinkDescription<Self::View>) -> Result<Self::View, String> {
        // TODO: remove the unwrap call
        Ok(self.0.props.render_links.as_ref().unwrap()(
            self.0.scope,
            link,
        ))
    }

    fn has_custom_component(self, name: &str) -> bool {
        self.0.props.components.0.get(name).is_some()
    }

    fn render_custom_component(
        self,
        name: &str,
        input: rust_web_markdown::MdComponentProps<Self::View>,
    ) -> Result<Self::View, ComponentCreationError> {
        let f = self.0.props.components.0.get(name).unwrap();
        f(self.0.scope, input)
    }
}

#[allow(non_snake_case)]
pub fn Markdown<'a>(cx: &'a Scoped<MdProps<'a>>) -> Element<'a> {
    let context = MdContext(cx);
    render_markdown(context, cx.props.src)
}
