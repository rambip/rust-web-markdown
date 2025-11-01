use dioxus::prelude::*;

use dioxus_markdown::debug::DEBUG_INFO;
use dioxus_markdown::{Markdown, Options};

#[component]
fn Logger() -> Element {
    rsx! {
        {DEBUG_INFO.read().iter().map(|x| rsx! {
            li { "{x}" }
        })}
    }
}

#[component]
fn App() -> Element {
    let mut content = use_signal(|| String::from("**bold**"));
    let mut wikilinks_enabled = use_signal(|| false);
    let mut hardbreaks_enabled = use_signal(|| false);
    let mut debug_enabled = use_signal(|| false);

    let parse_options_default = Options::ENABLE_GFM
        | Options::ENABLE_MATH
        | Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS;

    let parse_options = use_memo(move || {
        if wikilinks_enabled() {
            parse_options_default
        } else {
            parse_options_default | Options::ENABLE_WIKILINKS
        }
    });

    rsx! {
        h1 { "Markdown Editor" }
        div { class: "container",
            div {
                textarea {
                    value: "{content}",
                    rows: "30",
                    oninput: move |evt| content.set(evt.value()),
                }
                div {
                    label { r#for: "wiki", "enable wikilinks" }
                    input {
                        r#type: "checkbox",
                        id: "wiki",
                        oninput: move |e| wikilinks_enabled.set(e.value() == "true"),
                    }
                }
                div {
                    label { r#for: "hardbreaks", "convert soft breaks to hard breaks" }
                    input {
                        r#type: "checkbox",
                        id: "hardbreaks",
                        oninput: move |e| hardbreaks_enabled.set(e.value() == "true"),
                    }
                }
                div {
                    label { r#for: "debug", "enable debugging" }
                    input {
                        r#type: "checkbox",
                        id: "debug",
                        oninput: move |e| debug_enabled.set(e.value() == "true"),
                    }
                }
            }
            div { class: "md-view",
                Markdown {
                    src: content,
                    wikilinks: wikilinks_enabled(),
                    hard_line_breaks: hardbreaks_enabled(),
                    parse_options: parse_options(),
                }
            }
            div { class: "debug-view",
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
