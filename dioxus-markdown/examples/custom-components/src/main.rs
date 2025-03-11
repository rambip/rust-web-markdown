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
    }
}

#[component]
fn ColorBox(children: Element) -> Element {
    rsx! {
        div{
            style: "border: 2px solid blue",
            {children}
        }
    }
}

// create a component that renders a div with the text "Hello, world!"
#[component]
fn App() -> Element {
    let mut components = CustomComponents::new();

    components.register("Counter", |props| {
        Ok(rsx! {
            Counter {initial: props.get_parsed_optional("initial")?.unwrap_or(0)}
        })
    });

    components.register("box", |props| {
        let children = props.children;
        Ok(rsx! {
            ColorBox {children}
        })
    });

    rsx! {
        h1 {"Source"}
        Markdown {
            src: "```md\n{MARKDOWN_SOURCE}\n``"
        }

        h1 {"Result"}
        Markdown {
            src: MARKDOWN_SOURCE,
            components: components
        }
    }
}

fn main() {
    // launch the web app
    dioxus::launch(App);
}
