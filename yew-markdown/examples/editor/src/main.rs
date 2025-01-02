use yew::prelude::*;
use yew_markdown::Markdown;
mod input;
use input::TextArea;

struct App {
    content: String,
}

enum Msg {
    UpdateContent(String),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateContent(s) => self.content = s,
        }
        true
    }

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            content: String::new(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let oninput = ctx.link().callback(|s| Msg::UpdateContent(s));

        html! {
            <div style={"display: flex; align-items: top;"}>
                <TextArea placeholder={"enter markdown here"} oninput={oninput}
                    rows={80} cols={50}
                    style={"margin: 20px"}
                />
                <Markdown src={self.content.clone()} wikilinks=true/>
            </div>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
