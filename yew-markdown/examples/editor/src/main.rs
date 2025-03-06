use yew::prelude::*;
use yew_markdown::Markdown;
mod input;
use input::{get_value_from_checkbox, TextArea};

struct App {
    content: String,
    hard_line_breaks: bool,
}

enum Msg {
    UpdateContent(String),
    SetHardLineBreaks(bool),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetHardLineBreaks(b) => self.hard_line_breaks = b,
            Msg::UpdateContent(s) => self.content = s,
        }
        true
    }

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            content: String::new(),
            hard_line_breaks: false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let oninput = ctx.link().callback(|s| Msg::UpdateContent(s));
        let oninput_checkbox = ctx
            .link()
            .callback(|s: InputEvent| Msg::SetHardLineBreaks(get_value_from_checkbox(s)));

        html! {
            <div style={"display: flex; align-items: top;"}>
                <TextArea placeholder={"enter markdown here"} oninput={oninput}
                    rows={80} cols={50}
                    style={"margin: 20px"}
                />
                <div>
                <label for="hard_line_breaks">{"convert soft breaks to hard breaks"}</label>
                <input
                    id="hard_line_breaks"
                    type="checkbox"
                    oninput={oninput_checkbox}
                    />
                </div>
                <div>
                    <Markdown
                        src={self.content.clone()}
                        hard_line_breaks={self.hard_line_breaks}
                        />
                </div>
            </div>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
