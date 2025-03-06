#![allow(non_snake_case)]

use dioxus::prelude::*;

use dioxus_markdown::debug::DEBUG_INFO;
use dioxus_markdown::Markdown;

#[component]
fn Logger() -> Element {
    rsx! {
        {
            DEBUG_INFO.read().iter().map(|x| rsx! {li {"{x}"}})
        }
    }
}

fn App() -> Element {
    let mut content = use_signal(|| String::from("**bold**"));
    let mut wikilinks_enabled = use_signal(|| false);
    let mut hardbreaks_enabled = use_signal(|| false);
    let mut debug_enabled = use_signal(|| false);

    rsx! {
        h1 {"Markdown Editor"},
        div {
            class: "container",
            div {
                textarea {
                    value: "{content}",
                    rows: "30",
                    oninput: move |evt| content.set(evt.value()),
                },
                div {
                    label { r#for: "wiki", "enable wikilinks" },
                    input {r#type: "checkbox", id: "wiki",
                        oninput: move |e| wikilinks_enabled.set(e.value()=="true")
                    }
                }
                div {
                    label { r#for: "hardbreaks", "convert soft breaks to hard breaks" },
                    input {r#type: "checkbox", id: "hardbreaks",
                        oninput: move |e| hardbreaks_enabled.set(e.value()=="true")
                    }
                }
                div {
                    label { r#for: "debug", "enable debugging" },
                    input {r#type: "checkbox", id: "debug",
                        oninput: move |e| debug_enabled.set(e.value()=="true")
                    }
                }
            },
            div {
                class: "md-view",
                Markdown {
                    src: content,
                    wikilinks: wikilinks_enabled(),
                    hard_line_breaks: hardbreaks_enabled(),
                },
            }
            div {
                class: "debug-view",
                if debug_enabled() {
                    Logger {}
                }
            }
        }
    }
}

fn main() {
    dioxus::launch(App)
}
