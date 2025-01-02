use leptos::*;
use leptos_markdown::{Markdown, MarkdownMouseEvent};

static MARKDOWN_SOURCE : &str = r#"
# Interactive markdown experiment
## Goal
This page illustrates how you can use the `onclick` property of the `Markdown` component in order to add some interactivity in your markdown

## Usage
Test for yourself: click on any text on this page and it will appear highlighted in the source


# Click me

| title 1 | title 2 |
| ------- | ------- |
| **bold**|  `code` |

$$\sum_{n=-\infty}^{+\infty}c_n e^{inx}$$

> Quote

"#;


#[component]
fn App() -> impl IntoView {
    let (position, set_position) = create_signal(0..0);

    let onclick = move |e: MarkdownMouseEvent| set_position(e.position);

    let before = move || &MARKDOWN_SOURCE[0..position().start];
    let middle = move || &MARKDOWN_SOURCE[position()];
    let after =  move || &MARKDOWN_SOURCE[position().end..];

    view!{
        <div>
            <Markdown src=MARKDOWN_SOURCE on_click=onclick/>
                <br/>
                <hr/>
                <p>{"markdown source:"}</p>
                <pre style={"border: 2px solid orange"}>
                {before}
                <span style={"background-color: orange"}>{middle}</span>
                {after}
                </pre>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App)
}
