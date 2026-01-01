use yew::prelude::*;
use yew_markdown::Markdown;
mod input;
use input::{get_value_from_checkbox, TextArea};

struct App {
    content: String,
    hard_line_breaks: bool,
    debug_info: Vec<String>,
}

enum Msg {
    UpdateContent(String),
    UpdateDebugInfo(Vec<String>),
    SetHardLineBreaks(bool),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetHardLineBreaks(b) => self.hard_line_breaks = b,
            Msg::UpdateContent(s) => self.content = s,
            Msg::UpdateDebugInfo(s) => {
                let need_render = self.debug_info != s;
                self.debug_info = s;
                return need_render;
            }
        }
        true
    }

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            content: String::new(),
            hard_line_breaks: false,
            debug_info: Vec::new(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let oninput = ctx.link().callback(Msg::UpdateContent);
        let oninput_checkbox = ctx
            .link()
            .callback(|s: InputEvent| Msg::SetHardLineBreaks(get_value_from_checkbox(s)));

        let send_debug_info = ctx.link().callback(Msg::UpdateDebugInfo);
        let debug_info = self.debug_info.iter().map(|x| html! {<li>{x}</li>});

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
                        send_debug_info={send_debug_info}
                        />
                </div>
                <div>
                    <h3>{"Syntax tree"}</h3>
                    {for debug_info}
                </div>
            </div>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
