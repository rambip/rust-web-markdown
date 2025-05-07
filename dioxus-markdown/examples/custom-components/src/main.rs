use dioxus::prelude::*;
use dioxus_markdown::*;

static MARKDOWN_SOURCE: &str = r#"
## Here is a counter:
<Counter initial="5"/>

<Counter initial="a"/>

<Counter/>

## Here is a Box:
<box>

**I am in a blue box !**

</box>
"#;

#[component]
fn Counter(initial: i32) -> Element {
    let mut count = use_signal(|| initial);

    rsx! {
        div {
            button { onclick: move |_| count -= 1, "-" }
            "{count}"
            button { onclick: move |_| count += 1, "+" }
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

    components.register("Counter", |props| {
        Ok(rsx! {
            Counter { initial: props.get_parsed_optional("initial")?.unwrap_or(0) }
        })
    });

    components.register("box", |props| {
        let children = props.children;
        Ok(rsx! {
            ColorBox { children }
        })
    });

    rsx! {
        h1 { "Source" }
        Markdown { src: "```md\n{MARKDOWN_SOURCE}\n``" }

        h1 { "Result" }
        Markdown { src: MARKDOWN_SOURCE, components }
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

    // From https://dioxuslabs.com/learn/0.6/cookbook/testing/
    fn assert_rsx_eq(first: Element, second: Element) {
        let first = dioxus_ssr::render_element(first);
        let second = dioxus_ssr::render_element(second);
        pretty_assertions::assert_str_eq!(first, second);
    }

    // Adapted from https://dioxuslabs.com/learn/0.6/cookbook/testing/
    fn test_hook_simple(mut check: impl FnMut() + 'static) {
        fn mock_app() -> Element {
            rsx! {
                div {}
            }
        }

        let vdom = VirtualDom::new(mock_app);

        vdom.in_runtime(|| {
            ScopeId::ROOT.in_runtime(|| {
                check();
            })
        })
    }

    #[test]
    fn minimal() {
        test_hook_simple(|| {
            assert_rsx_eq(
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
        components
    }

    #[test]
    fn custom() {
        test_hook_simple(|| {
            assert_rsx_eq(
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
            assert_rsx_eq(
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
            assert_rsx_eq(
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
            assert_rsx_eq(
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
            assert_rsx_eq(
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
            assert_rsx_eq(
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
            assert_rsx_eq(
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
            assert_rsx_eq(
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
            assert_rsx_eq(
                rsx! {
                    // TODO: provide a way to opt into preserving html as text
                    Markdown {
                        src: "For some values of X, Y, and Z, assume X<Y and Y>Z",
                        components: components(),
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
            assert_rsx_eq(
                rsx! {
                    Markdown {
                        src: "For some values of X, Y, and Z, assume X<Y and Y>Z",
                        components: components(),
                    }
                },
                rsx! {
                    p { style: "", class: "",
                        span { style: "", class: "", "For some values of X, Y, and Z, assume X" }
                        span { style: "", class: "", dangerous_inner_html: "<Y and Y>" }
                        span { style: "", class: "", "Z" }
                    }
                },
            )
        });
    }
}
