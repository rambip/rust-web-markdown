#![allow(non_snake_case)]
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
fn Counter(cx: Scope, initial: i32) -> Element<'a> {
    let mut count = use_state(cx, || *initial);

    cx.render(rsx! {
        div{
            button {
                onclick: move |_| count-=1,
                "-"
            },
            "{count}",
            button {
                onclick: move |_| count+=1,
                "+"
            }
        }
    })
}

#[component]
fn ColorBox<'a>(cx: Scope, children: Element<'a>) -> Element<'a> {
    cx.render(rsx! {
        div{
            style: "border: 2px solid blue",
            children
        }
    })
}

// create a component that renders a div with the text "Hello, world!"
fn App(cx: Scope) -> Element {
    let mut components = CustomComponents::new();

    components.register("Counter", |cx, props| {
        Ok(render! {
            Counter {initial: props.get_parsed_optional("initial")?.unwrap_or(0)}
        })
    });

    components.register("box", |cx, props| {
        Ok(render! {
            ColorBox {props.children}
        })
    });

    cx.render(rsx! {
        h1 {"Source"}
        Markdown {
            src: "```md\n{MARKDOWN_SOURCE}\n``"
        }

        h1 {"Result"}
        Markdown {
            src: MARKDOWN_SOURCE,
            components: components
        }
    })
}

fn main() {
    // launch the web app
    dioxus_web::launch(App);
}
