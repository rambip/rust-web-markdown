use yew::prelude::*;
use yew_markdown::{Markdown, MarkdownMouseEvent};

use core::ops::Range;

static MARKDOWN_SOURCE: &str = r#"
# Interactive markdown experiment
## Goal
This page illustrates how you can use the `onclick` property of the `Markdown` component in order to add some interactivity in your markdown

## Usage
Test for yourself: click on any text on this page and it will appear highlighted in the source



## Code

Here is how you can use it in your project:
```rust

let callback = ctx.link().callback(|x: MarkdownMouseEvent| 
                                   Msg::ShowSource(x.start_position, x.end_position)
<Markdown src={MARKDOWN_SOURCE} caching=false onclick={callback}/>
```
"#;

/// `start_index` represents an index in the markdown source,
/// `end_index` too.
/// The range between the two is a specific region of text in the markdown source.
struct App {
    start_index: usize,
    end_index: usize,
}

enum Msg {
    ShowSource(Range<usize>),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ShowSource(range) => {
                self.start_index = range.start;
                self.end_index = range.end;
            }
        }
        true
    }

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            start_index: 0,
            end_index: 0,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let (before, x) = MARKDOWN_SOURCE.split_at(self.start_index);
        let (middle, after) = x.split_at(self.end_index - self.start_index);

        let callback = ctx
            .link()
            .callback(|x: MarkdownMouseEvent| Msg::ShowSource(x.position));
        html! {
            <div>
                <Markdown src={MARKDOWN_SOURCE} onclick={callback}/>
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
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
