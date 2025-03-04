#![allow(non_snake_case)]


use dioxus::prelude::*;

use dioxus_markdown::Markdown;
use dioxus_markdown::debug::EventInfo;

#[component]
fn Logger(cx: Scope) -> Element {
    let debug_info = use_shared_state::<EventInfo>(cx).unwrap();
    render!{
        debug_info.read().0.iter().map(|x: &String| cx.render(rsx!{li {x.clone()}}))
    }
}


fn App(cx: Scope) -> Element {
    let content = use_state(cx, || String::from("**bold**"));
    let wikilinks_enabled = use_state(cx, || false);
    let hardbreaks_enabled = use_state(cx, || false);
    let debug_enabled = use_state(cx, || false);

    use_shared_state_provider(cx, || EventInfo(vec![]));

    render!{
        h1 {"Markdown Editor"},
        div {
            class: "container",
            div {
                textarea {
                    value: "{content}",
                    rows: "30",
                    oninput: move |evt| content.set(evt.value.clone()),
                },
                div {
                    label { r#for: "wiki", "enable wikilinks" },
                    input {r#type: "checkbox", id: "wiki",
                        oninput: move |e| wikilinks_enabled.set(e.value=="true")
                    }
                }
                div {
                    label { r#for: "hardbreaks", "convert soft breaks to hard breaks" },
                    input {r#type: "checkbox", id: "hardbreaks",
                        oninput: move |e| hardbreaks_enabled.set(e.value=="true")
                    }
                }
                div {
                    label { r#for: "debug", "enable debugging" },
                    input {r#type: "checkbox", id: "debug",
                        oninput: move |e| debug_enabled.set(e.value=="true")
                    }
                }
            },
            div {
                class: "md-view",
                Markdown {
                    src: content,
                    wikilinks: *wikilinks_enabled.get(),
                    hard_line_breaks: *hardbreaks_enabled.get(),
                },
            }
            div {
                class: "debug-view",
                if **debug_enabled {
                    cx.render(rsx!{
                        Logger {}
                    })
                }
            }
        }
    }
}

fn main() {
    dioxus_web::launch(App)
}
