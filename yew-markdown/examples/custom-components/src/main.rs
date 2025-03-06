use yew::prelude::*;
use yew_markdown::{CustomComponents, Markdown};

static MARKDOWN_SOURCE: &str = r#"
## Here are a few counters:
<Counter initial="5"/>

<Counter/>

<Counter initial="a"/>

## Here is a Box:
<box>

**I am in a blue box !**

</box>
"#;

#[derive(PartialEq, Properties)]
struct CounterProps {
    initial: Option<i32>,
}

#[function_component]
fn Counter(props: &CounterProps) -> Html {
    let count = use_state(move || props.initial.unwrap_or(0));

    let increment = {
        let count = count.clone();
        Callback::from(move |_| count.set(*count + 1))
    };
    let decrement = {
        let count = count.clone();
        Callback::from(move |_| count.set(*count - 1))
    };

    html! {
        <div>
            <button onclick={decrement}>{"-"}</button>
            {*count}
            <button onclick={increment}>{"+"}</button>
        </div>
    }
}

#[derive(PartialEq, Properties)]
struct BoxProps {
    children: Children,
}

#[function_component]
fn BlueBox(props: &BoxProps) -> Html {
    html! {
        <div style="border: 2px solid blue">
            {props.children.clone()}
        </div>
    }
}

#[function_component(App)]
fn app() -> Html {
    let mut components = CustomComponents::new();
    components.register("Counter", |p| {
        Ok(html! {
        <Counter initial={p.get_parsed_optional("initial")?} />})
    });

    components.register("box", |p| Ok(html! {<BlueBox>{p.children}</BlueBox>}));

    html! {
        <div>
            <h1>{"The source"}</h1>
            <Markdown src={format!("```md\n{MARKDOWN_SOURCE}\n")}/>

            <h1>{"The result"}</h1>
            <Markdown src={MARKDOWN_SOURCE} components={components}/>
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
