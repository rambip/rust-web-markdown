use std::fmt::Display;

use dioxus::prelude::*;
use dioxus_markdown::*;

static MARKDOWN_SOURCE: &str = r#"
## Here is a counter:
<EphemeralCounter initial="5"/>

An invalid counter:
<EphemeralCounter initial="a"/>

A defaulted counter:
<EphemeralCounter/>

A counter which modifies the document:
<PersistedCounter value="5"/>

## Here is a Box:
<custom-box>

**I am in a blue box !**

</custom-box>
"#;

/// A counter who's current count is not stored in the document.
#[component]
fn EphemeralCounter(initial: i32) -> Element {
    let count = use_signal(|| initial);
    counter_inner_signal(count)
}

/// A counter who's current count is stored in the document.
#[component]
fn PersistedCounter(count: ReadWriteBox<i32>) -> Element {
    counter_inner_signal(count)
}

/// Internals of counter, which can be provided the count in a signal like value.
fn counter_inner_signal<T>(mut count: T) -> Element
where
    T: Clone + Display + std::ops::SubAssign<i32> + std::ops::AddAssign<i32> + 'static,
{
    // This supports non-copy values to support ReadWriteBox, so unlike with Signal, a clone is needed here.
    let mut count2 = count.clone();
    rsx! {
        div {
            button { onclick: move |_| count -= 1, "-" }
            "{count}"
            button { onclick: move |_| count2 += 1, "+" }
        }
    }
}

#[component]
fn ColorBox(children: Element) -> Element {
    rsx! {
        div { style: "border: 2px solid blue", {children} }
    }
}

#[component]
fn App() -> Element {
    let mut components = CustomComponents::new();

    let src = use_signal(|| MARKDOWN_SOURCE.to_string());

    components.register("EphemeralCounter", |props| {
        Ok(rsx! {
            EphemeralCounter { initial: props.get_parsed_optional("initial")?.unwrap_or(0) }
        })
    });

    components.register("PersistedCounter", move |props| {
        let value = props.get_attribute("value").unwrap();
        let count = ReadWriteBox::from_sub_string(src, value.range)?;
        Ok(rsx! {
            PersistedCounter { count }
        })
    });

    components.register("custom-box", |props| {
        let children = props.children;
        Ok(rsx! {
            ColorBox { children }
        })
    });

    rsx! {
        h1 { "Source" }
        Markdown { src: "```md\n{src}\n```" }

        h1 { "Result" }
        Markdown { src: src, components }
    }
}

fn main() {
    // launch the web app
    dioxus::launch(App);
}

/// If default target is set to wasm in .cargo/config.toml, these need a specific target provided to run them
/// for example `cargo test --target "x86_64-unknown-linux-gnu"`
#[cfg(test)]
mod tests {
    use dioxus::prelude::*;
    use dioxus_markdown::{CustomComponents, Markdown};

    // Adapted From https://dioxuslabs.com/learn/0.7/guides/testing/web
    // Using a macro makes the printed error location in output nicer.
    macro_rules! assert_rsx_eq {
        ($left:expr, $right:expr $(,)?) => {{
            let first = dioxus_ssr::render_element($left);
            let second = dioxus_ssr::render_element($right);
            ::pretty_assertions::assert_str_eq!(first, second);
        }};
    }

    // Adapted from https://dioxuslabs.com/learn/0.7/guides/testing/web
    fn test_hook_simple(mut check: impl FnMut() + 'static) {
        fn mock_app() -> Element {
            rsx! {
                div {}
            }
        }

        let vdom = VirtualDom::new(mock_app);

        vdom.in_scope(ScopeId::ROOT, || {
            check();
        });
    }

    #[test]
    fn minimal() {
        test_hook_simple(|| {
            assert_rsx_eq!(
                rsx! {
                    Markdown { src: "ZZZ" }
                },
                rsx! {
                    p { style: "", class: "",
                        span { style: "", class: "", "ZZZ" }
                    }
                },
            )
        });
    }

    /// Must be run in a Dioxus runtime
    fn components() -> CustomComponents {
        let mut components = CustomComponents::new();
        components.register("X", |_props| Ok(rsx! { "Content" }));
        components.register("Place", |props| {
            let test = props.get_attribute("test").unwrap();
            let range = test.range;
            Ok(rsx! { "{range.start},{range.end}" })
        });
        components
    }

    #[test]
    fn custom() {
        test_hook_simple(|| {
            assert_rsx_eq!(
                rsx! {
                    Markdown { src: "<X/>", components: components() }
                },
                // TODO: it seems a bit odd that Markdown wraps text in a `p` tag and a span, but doesn't do so when its just a custom component.
                // TO be more consistent with the below case, maybe everything should always be wrapped in a `p`?
                rsx! { "Content" },
            )
        });
    }

    #[test]
    fn custom_non_closing() {
        test_hook_simple(|| {
            assert_rsx_eq!(
                rsx! {
                    Markdown { src: "<X>", components: components() }
                },
                // TODO: A non self closing tag behaves the same as a self closing on when using a custom component.
                // It's not clear what syntaxes are supposed to be allowed for custom components (TODO: this should be documented somewhere).
                rsx! { "Content" },
            )
        });
    }

    #[test]
    fn custom_plus_text() {
        test_hook_simple(|| {
            assert_rsx_eq!(
                rsx! {
                    Markdown { src: "z<X/>", components: components() }
                },
                rsx! {
                    p { style: "", class: "",
                        span { style: "", class: "", "z" }
                        "Content"
                    }
                },
            )
        });
    }

    #[test]
    fn custom_plus_custom() {
        test_hook_simple(|| {
            assert_rsx_eq!(
                rsx! {
                    Markdown { src: "<X/><X/>", components: components() }
                },
                rsx! {
                    p { style: "", class: "", "ContentContent" }
                },
            )
        });
    }

    #[test]
    fn custom_line_custom() {
        test_hook_simple(|| {
            assert_rsx_eq!(
                rsx! {
                    Markdown { src: "<X/>\n<X/>", components: components() }
                },
                rsx! { "ContentContent" },
            )
        });
    }

    #[test]
    fn tag_plus_text() {
        test_hook_simple(|| {
            assert_rsx_eq!(
                rsx! {
                    Markdown { src: "z<X>", components: components() }
                },
                rsx! {
                    p { style: "", class: "",
                        span { style: "", class: "", "z" }
                        "Content"
                    }
                },
            )
        });
    }

    #[test]
    fn tag_plus_tag() {
        test_hook_simple(|| {
            assert_rsx_eq!(
                rsx! {
                    Markdown { src: "<X><X>", components: components() }
                },
                // TODO: this seems like it should either produce two Xs or error, but just gives 1
                rsx! {
                    p { style: "", class: "", "Content" }
                },
            )
        });
    }

    #[test]
    fn tag_line_tag() {
        test_hook_simple(|| {
            assert_rsx_eq!(
                rsx! {
                    Markdown { src: "<X>\n<X>", components: components() }
                },
                // TODO: this seems like it should either produce two Xs or error, but just gives 1
                rsx! { "Content" },
            )
        });
    }

    #[test]
    fn inline_html_like_as_text() {
        test_hook_simple(|| {
            assert_rsx_eq!(
                rsx! {
                    Markdown {
                        src: "For some values of X, Y, and Z, assume X<Y and Y>Z",
                        components: components(),
                        preserve_html: false,
                    }
                },
                rsx! {
                    p { style: "", class: "",
                        span { style: "", class: "", "For some values of X, Y, and Z, assume X" }
                        span { style: "", class: "", "<Y and Y>" }
                        span { style: "", class: "", "Z" }
                    }
                },
            )
        });
    }

    #[test]
    fn inline_html_like_as_html() {
        test_hook_simple(|| {
            assert_rsx_eq!(
                rsx! {
                    Markdown {
                        src: "For some values of X, Y, and Z, assume X<Y and Y>Z",
                        components: components(),
                        preserve_html: true,
                    }
                },
                rsx! {
                    p { style: "", class: "",
                        span { style: "", class: "", "For some values of X, Y, and Z, assume X" }
                        span {
                            style: "",
                            class: "",
                            dangerous_inner_html: "<Y and Y>",
                        }
                        span { style: "", class: "", "Z" }
                    }
                },
            )
        });
    }

    #[test]
    fn parameter_range() {
        static SRC: &'static str = " <Place  test=\"abc\"/>";
        test_hook_simple(move || {
            let expected = 15..18;
            assert_eq!(&SRC[expected.clone()], "abc");
            assert_rsx_eq!(
                rsx! {
                    Markdown { src: SRC, components: components() }
                },
                rsx! {span {style: "", class: "", " "} "{expected.start},{expected.end}"},
            )
        });
    }

    #[test]
    fn component_with_proper_closing() {
        // Test a component with both opening and closing tags with content between
        test_hook_simple(|| {
            let mut components = CustomComponents::new();
            components.register("X", |props| {
                let children = props.children;
                Ok(rsx! { div { {children} } })
            });
            assert_rsx_eq!(
                rsx! {
                    Markdown { src: "<X>\ntext\n</X>", components }
                },
                rsx! {
                    div {
                        span { style: "", class: "", "text\n" }
                    }
                },
            )
        });
    }

    #[test]
    fn block_component_with_attributes() {
        // Test block-level component with attributes (no leading space)
        test_hook_simple(|| {
            let src = "<Place test=\"abc\"/>\n";
            let expected = 13..16;
            assert_eq!(&src[expected.clone()], "abc");
            assert_rsx_eq!(
                rsx! {
                    Markdown { src: src, components: components() }
                },
                rsx! { "{expected.start},{expected.end}" },
            )
        });
    }

    #[test]
    fn inline_component_in_paragraph() {
        // Test inline component within a paragraph (not block-level)
        test_hook_simple(|| {
            assert_rsx_eq!(
                rsx! {
                    Markdown { src: "text <X/> more", components: components() }
                },
                rsx! {
                    p { style: "", class: "",
                        span { style: "", class: "", "text " }
                        "Content"
                        span { style: "", class: "", " more" }
                    }
                },
            )
        });
    }

    #[test]
    fn multiple_inline_components_in_paragraph() {
        // Test multiple inline components in the same paragraph
        test_hook_simple(|| {
            assert_rsx_eq!(
                rsx! {
                    Markdown { src: "text <X/> middle <X/> end", components: components() }
                },
                rsx! {
                    p { style: "", class: "",
                        span { style: "", class: "", "text " }
                        "Content"
                        span { style: "", class: "", " middle " }
                        "Content"
                        span { style: "", class: "", " end" }
                    }
                },
            )
        });
    }

    #[test]
    fn three_components_on_separate_lines() {
        // Test three components on separate lines to ensure End(HtmlBlock) handling is correct
        test_hook_simple(|| {
            assert_rsx_eq!(
                rsx! {
                    Markdown { src: "<X/>\n<X/>\n<X/>", components: components() }
                },
                rsx! { "ContentContentContent" },
            )
        });
    }

    #[test]
    fn component_followed_by_paragraph() {
        // Test that a block-level component followed by text works correctly
        test_hook_simple(|| {
            assert_rsx_eq!(
                rsx! {
                    Markdown { src: "<X/>\n\nParagraph text", components: components() }
                },
                rsx! {
                    "Content"
                    p { style: "", class: "",
                        span { style: "", class: "", "Paragraph text" }
                    }
                },
            )
        });
    }

    #[test]
    fn paragraph_followed_by_component() {
        // Test that a paragraph followed by a block-level component works correctly
        test_hook_simple(|| {
            assert_rsx_eq!(
                rsx! {
                    Markdown { src: "Paragraph text\n\n<X/>", components: components() }
                },
                rsx! {
                    p { style: "", class: "",
                        span { style: "", class: "", "Paragraph text" }
                    }
                    "Content"
                },
            )
        });
    }

    #[test]
    fn component_with_markdown_content() {
        // Test component containing markdown content
        test_hook_simple(|| {
            let mut components = CustomComponents::new();
            components.register("X", |props| {
                let children = props.children;
                Ok(rsx! { div { class: "box", {children} } })
            });
            assert_rsx_eq!(
                rsx! {
                    Markdown { src: "<X>\n\n**bold text**\n\n</X>", components }
                },
                rsx! {
                    div { class: "box",
                        p { style: "", class: "",
                            b { style: "", class: "",
                                span { style: "", class: "", "bold text" }
                            }
                        }
                    }
                },
            )
        });
    }

    #[test]
    fn mixed_block_and_inline_components() {
        // Test mixing block-level and inline components
        test_hook_simple(|| {
            assert_rsx_eq!(
                rsx! {
                    Markdown { src: "<X/>\n\ntext <X/> more", components: components() }
                },
                rsx! {
                    "Content"
                    p { style: "", class: "",
                        span { style: "", class: "", "text " }
                        "Content"
                        span { style: "", class: "", " more" }
                    }
                },
            )
        });
    }
}
