#![allow(non_snake_case)]

use dioxus::prelude::*;

use dioxus_markdown::*;

static MARKDOWN_SOURCE: &str = r#"
# Interactive markdown experiment
## Goal
This page illustrates how you can use the `onclick` property of the `Markdown` component in order to add some interactivity in your markdown

## Usage
Test for yourself: click on any text on this page and it will appear highlighted in the source



## Code

Here is how you can use it in your project:
```rust

let range = use_state(|| 0..0);

render!{
    Markdown {src: source, on_click: move |e: MarkdownMouseEvent|
        range.set(e.position)
    }
}
```
"#;

fn App() -> Element {
    let mut range = use_signal(|| 0..0);

    let (before, x) = MARKDOWN_SOURCE.split_at(range().start);
    let (middle, after) = x.split_at(range().len());

    let onclick = move |e: MarkdownMouseEvent| range.set(e.position);
    rsx! {
        div {
            Markdown {
                src:MARKDOWN_SOURCE,
                on_click:onclick
            }
            br {}
            hr {}
            pre {
                style: "border: 2 px solid orange",
                {before}
                span {
                    style: "background-color: orange",
                    {middle}
                }
                {after}
            }
        }
    }
}

fn main() {
    dioxus::launch(App)
}
