use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlTextAreaElement;
use yew::{function_component, prelude::*};

pub fn get_value_from_textarea(e: InputEvent) -> String {
    let event: Event = e.dyn_into().unwrap_throw();
    let event_target = event.target().unwrap_throw();
    let target: HtmlTextAreaElement = event_target.dyn_into().unwrap_throw();
    target.value()
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub placeholder: AttrValue,
    pub oninput: Callback<String>,
    pub cols: Option<u32>,
    pub rows: Option<u32>,
    pub style: Option<AttrValue>,
}

#[function_component]
pub fn TextArea(props: &Props) -> Html {
    let oninput = props.oninput.clone();
    let callback = Callback::from(move |e: InputEvent| oninput.emit(get_value_from_textarea(e)));
    html! {
        <textarea
            placeholder={props.placeholder.clone()}
            oninput={callback}
            cols={props.cols.map(|x| x.to_string())}
            rows={props.rows.map(|x| x.to_string())}
            style={props.style.clone()}
            />
    }
}
