use yew::prelude::*;
use yew_markdown::Markdown;
mod content;

struct App {
    index: Option<usize>,
    markdown_content: Vec<String>,
}

enum Msg {
    Next,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        let n = self.markdown_content.len();
        match (msg, self.index) {
            (Msg::Next, Some(i)) if i < n - 1 => self.index = Some(i + 1),
            (Msg::Next, Some(i)) if i == n - 1 => self.index = None,
            _ => (),
        }
        true
    }

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            index: Some(0),
            markdown_content: content::generate_content(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|_| Msg::Next);
        match self.index {
            Some(i) => html! {
                <div>
                    <button onclick={onclick}> {"next"}</button>
                    <Markdown src={self.markdown_content[i].clone()}/>
                </div>
            },
            None => html! {"the end"},
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
