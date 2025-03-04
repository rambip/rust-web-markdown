use leptos::*;
use leptos_markdown::*;

// macro_rules! dbg {
//     ($var:expr) => {{
//         let x = $var;
//         leptos::logging::log!("{:?}", x);
//         x
//     }}
// }


#[component]
pub fn SimpleCounter(initial_value: i32) -> impl IntoView {
    // create a reactive signal with the initial value
    let (value, set_value) = create_signal(initial_value);

    // create event handlers for our buttons
    // note that `value` and `set_value` are `Copy`, so it's super easy to move them into closures
    let clear = move |_| set_value(0);
    let decrement = move |_| set_value.update(|value| *value -= 1);
    let increment = move |_| set_value.update(|value| *value += 1);

    // create user interfaces with the declarative `view!` macro
    view! {
        <div>
            <button on:click=clear>Clear</button>
            <button on:click=decrement>-1</button>
            // text nodes can be quoted or unquoted
            <span>"Value: " {value} "!"</span>
            <button on:click=increment>+1</button>
        </div>
    }
}

#[component]
fn BlueBox(children: Children) -> impl IntoView {
    view!{
        <div style="border: 2px solid blue">
            {children()}
        </div>
    }
}

static MARKDOWN_SOURCE: &'static str = r#"
## Here are a few counters:
<Counter initial="5"/>

<Counter/>

<Counter initial="a"/>

## Here is a Box:
<box>

**I am in a blue box !**

</box>
"#;

#[component]
fn App() -> impl IntoView {
    let mut components = CustomComponents::new();

    components.register("Counter", 
        |props| Ok(view!{
            <SimpleCounter initial_value=props.get_parsed_optional("initial")?.unwrap_or(0)/>
        })
    );

    components.register("box", 
        |props| Ok(view!{
            <BlueBox>{props.children}</BlueBox>
        })
    );

    view!{
        <h1>"The source"</h1>
        <Markdown
            src=format!("```md\n{MARKDOWN_SOURCE}\n")
        />
        <br/>
        <h1>"The result"</h1>
        <Markdown
            components=components
            src=MARKDOWN_SOURCE
        />
    }
}


fn main(){
    mount_to_body(App)
}
