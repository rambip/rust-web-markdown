use web_framework_markdown::{
    markdown_component, Context, CowStr, ElementAttributes, HtmlElement, MarkdownProps, StyleLink,
};

use core::ops::Range;

use std::collections::BTreeMap;

pub use web_framework_markdown::{ComponentCreationError, LinkDescription, Options};

use yew::prelude::{
    function_component, html, AttrValue, Callback, Html, Properties, UseStateHandle,
};

pub type MdComponentProps = web_framework_markdown::MdComponentProps<Html>;

use web_sys::{window, MouseEvent};

#[derive(Clone, Debug)]
pub struct MarkdownMouseEvent {
    /// the original mouse event triggered when a text element was clicked on
    pub mouse_event: MouseEvent,

    /// The corresponding range in the markdown source, as a slice of [u8].
    pub position: Range<usize>,
    // TODO: add a clonable tag for the type of the element
    // pub tag: pulldown_cmark::Tag<'a>,
}

/// component store.
/// It is called when there is a `<CustomComponent>` inside the markdown source.
/// It is basically a hashmap but more efficient for a small number of items
#[derive(PartialEq, Clone, Default)]
pub struct CustomComponents(
    BTreeMap<&'static str, Callback<MdComponentProps, Result<Html, ComponentCreationError>>>,
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
        F: Fn(MdComponentProps) -> Result<Html, ComponentCreationError> + 'static,
    {
        self.0.insert(name, Callback::from(component));
    }
}

impl<'a> Context<'a, 'static> for &'a Props {
    type View = Html;

    type Handler<T: 'static> = Callback<T>;

    type MouseEvent = MouseEvent;

    fn props(self) -> MarkdownProps {
        let Props {
            theme,
            wikilinks,
            hard_line_breaks,
            parse_options,
            ..
        } = self;

        MarkdownProps {
            theme: *theme,
            wikilinks: *wikilinks,
            hard_line_breaks: *hard_line_breaks,
            parse_options: *parse_options,
        }
    }

    #[cfg(feature = "debug")]
    fn send_debug_info(self, info: Vec<String>) {
        if let Some(sender) = &self.send_debug_info {
            sender.emit(info)
        }
    }

    fn el_with_attributes(
        self,
        e: HtmlElement,
        inside: Self::View,
        attributes: ElementAttributes<Callback<MouseEvent>>,
    ) -> Self::View {
        let style = attributes.style.map(|x| x.to_string());
        let classes: Vec<_> = attributes.classes.iter().map(|x| x.to_string()).collect();
        let on_click = attributes.on_click;

        match e {
            HtmlElement::Div => {
                html! {<div style={style} onclick={on_click} class={classes}>{inside}</div>}
            }
            HtmlElement::Span => {
                html! {<span style={style} onclick={on_click} class={classes}>{inside}</span>}
            }
            HtmlElement::Paragraph => {
                html! {<p  style={style} onclick={on_click} class={classes}>{inside}</p>}
            }
            HtmlElement::Ul => {
                html! {<ul  style={style} onclick={on_click} class={classes}>{inside}</ul>}
            }
            HtmlElement::Ol(start) => {
                html! {<ol start={start.to_string()}  style={style} onclick={on_click} class={classes}>{inside}</ol>}
            }
            HtmlElement::Li => {
                html! {<li  style={style} onclick={on_click} class={classes}>{inside}</li>}
            }
            HtmlElement::BlockQuote => {
                html! {<blockquote  style={style} onclick={on_click} class={classes}>{inside}</blockquote>}
            }
            HtmlElement::Heading(1) => {
                html! {<h1  style={style} onclick={on_click} class={classes}>{inside}</h1>}
            }
            HtmlElement::Heading(2) => {
                html! {<h2  style={style} onclick={on_click} class={classes}>{inside}</h2>}
            }
            HtmlElement::Heading(3) => {
                html! {<h3  style={style} onclick={on_click} class={classes}>{inside}</h3>}
            }
            HtmlElement::Heading(4) => {
                html! {<h4  style={style} onclick={on_click} class={classes}>{inside}</h4>}
            }
            HtmlElement::Heading(5) => {
                html! {<h5  style={style} onclick={on_click} class={classes}>{inside}</h5>}
            }
            HtmlElement::Heading(6) => {
                html! {<h6  style={style} onclick={on_click} class={classes}>{inside}</h6>}
            }
            HtmlElement::Heading(_) => panic!(),
            HtmlElement::Table => {
                html! {<table  style={style} onclick={on_click} class={classes}>{inside}</table>}
            }
            HtmlElement::Thead => {
                html! {<thead  style={style} onclick={on_click} class={classes}>{inside}</thead>}
            }
            HtmlElement::Trow => {
                html! {<tr  style={style} onclick={on_click} class={classes}>{inside}</tr>}
            }
            HtmlElement::Tcell => {
                html! {<td  style={style} onclick={on_click} class={classes}>{inside}</td>}
            }
            HtmlElement::Italics => {
                html! {<i  style={style} onclick={on_click} class={classes}>{inside}</i>}
            }
            HtmlElement::Bold => {
                html! {<b  style={style} onclick={on_click} class={classes}>{inside}</b>}
            }
            HtmlElement::StrikeThrough => {
                html! {<s  style={style} onclick={on_click} class={classes}>{inside}</s>}
            }
            HtmlElement::Pre => {
                html! {<pre  style={style} onclick={on_click} class={classes}>{inside}</pre>}
            }
            HtmlElement::Code => {
                html! {<code  style={style} onclick={on_click} class={classes}>{inside}</code>}
            }
        }
    }

    fn el_span_with_inner_html(
        self,
        inner_html: String,
        attributes: ElementAttributes<Callback<MouseEvent>>,
    ) -> Self::View {
        let style = attributes.style.map(|x| x.to_string());
        let classes: Vec<_> = attributes.classes.iter().map(|x| x.to_string()).collect();
        let onclick = attributes.on_click;

        html! {
            <span style={style} onclick={onclick} class={classes}>
                {Html::from_html_unchecked(inner_html.into())}
            </span>
        }
    }

    fn el_hr(self, attributes: ElementAttributes<Callback<MouseEvent>>) -> Self::View {
        let style = attributes.style.map(|x| x.to_string());
        let classes: Vec<_> = attributes.classes.iter().map(|x| x.to_string()).collect();
        let on_click = attributes.on_click;
        html! {<hr  style={style} onclick={on_click} class={classes}/>}
    }

    fn el_br(self) -> Self::View {
        html! {<br/>}
    }

    fn el_fragment(self, children: Vec<Self::View>) -> Self::View {
        children.into_iter().collect()
    }

    fn el_a(self, children: Self::View, href: String) -> Self::View {
        html! {<a href={href.to_string()}>{children}</a>}
    }

    fn el_img(self, src: String, alt: String) -> Self::View {
        html! {<img src={src} alt={alt}/>}
    }

    fn el_text(self, text: CowStr<'a>) -> Self::View {
        html! {text}
    }

    fn el_input_checkbox(
        self,
        checked: bool,
        attributes: ElementAttributes<Callback<MouseEvent>>,
    ) -> Self::View {
        let style = attributes.style.map(|x| x.to_string());
        let classes: Vec<_> = attributes.classes.iter().map(|x| x.to_string()).collect();
        let on_click = attributes.on_click;
        html! {
            <input type="checkbox" checked={checked}
                onclick={on_click}
                class={classes}
                style={style}
            />
        }
    }

    fn call_handler<T: 'static>(callback: &Self::Handler<T>, input: T) {
        callback.emit(input)
    }

    fn make_md_handler(
        self,
        position: Range<usize>,
        stop_propagation: bool,
    ) -> Self::Handler<MouseEvent> {
        match &self.onclick {
            Some(f) => {
                let f = f.clone();
                Callback::from(move |e: MouseEvent| {
                    if stop_propagation {
                        e.stop_propagation()
                    }
                    let report = MarkdownMouseEvent {
                        mouse_event: e,
                        position: position.clone(),
                    };
                    f.emit(report)
                })
            }
            None => Callback::noop(),
        }
    }

    fn has_custom_links(self) -> bool {
        self.render_links.is_some()
    }

    fn render_links(self, link: LinkDescription<Html>) -> Result<Html, String> {
        let f = self.render_links.clone().unwrap();
        Ok(f.emit(link))
    }

    fn set_frontmatter(&mut self, frontmatter: String) {
        if let Some(setter) = &self.frontmatter {
            setter.set(frontmatter)
        }
    }

    fn has_custom_component(self, name: &str) -> bool {
        self.components.0.get(name).is_some()
    }

    fn render_custom_component(
        self,
        name: &str,
        input: MdComponentProps,
    ) -> Result<Self::View, ComponentCreationError> {
        let f = self.components.0.get(name).unwrap();
        f.emit(input)
    }
}

#[derive(PartialEq, Properties, Clone)]
pub struct Props {
    pub src: AttrValue,

    #[prop_or_default]
    pub onclick: Option<Callback<MarkdownMouseEvent, ()>>,

    #[prop_or_default]
    pub render_links: Option<Callback<LinkDescription<Html>, Html>>,

    #[prop_or_default]
    pub theme: Option<&'static str>,

    #[prop_or(false)]
    pub wikilinks: bool,

    #[prop_or(false)]
    pub hard_line_breaks: bool,

    #[prop_or_default]
    pub parse_options: Option<Options>,

    #[prop_or_default]
    pub components: CustomComponents,

    #[prop_or_default]
    pub frontmatter: Option<UseStateHandle<String>>,

    #[prop_or_default]
    pub send_debug_info: Option<Callback<Vec<String>>>,
}

#[function_component]
pub fn Markdown(props: &Props) -> Html {
    #[cfg(feature = "maths")]
    {
        let document = window().unwrap().document().unwrap();

        let link = document.create_element("link").unwrap();

        let StyleLink {
            rel,
            href,
            integrity,
            crossorigin,
        } = web_framework_markdown::MATH_STYLE_SHEET_LINK;
        link.set_attribute("rel", rel).unwrap();
        link.set_attribute("href", href).unwrap();
        link.set_attribute("integrity", integrity).unwrap();
        link.set_attribute("crossorigin", crossorigin).unwrap();

        document.head().unwrap().append_child(&link).unwrap();
    }

    markdown_component(props, &props.src)
}
