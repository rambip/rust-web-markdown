use leptos::*;
use leptos_markdown::{Markdown, debug::EventInfo};

#[component]
fn RenderZone(
              content: ReadSignal<String>,
              wikilinks_enabled: ReadSignal<bool>, 
              hard_breaks_enabled: ReadSignal<bool>,
              debug_mode: ReadSignal<bool>) -> impl IntoView {


    let (debug_info, set_debug_info) = create_signal(Vec::new());
    provide_context(EventInfo(set_debug_info));

    let debug_info_view = move || {
        debug_info()
            .iter()
            .map(|x| view!{<li>{x}</li>})
            .collect_view()
    };

    view!{
        <div style="border: 1px solid black; margin: 10px; width: 50%">
            <Markdown src=content 
                  wikilinks=wikilinks_enabled
                  hard_line_breaks=hard_breaks_enabled
            />
        </div>
        {move || debug_mode().then_some(
            view!{
                <ul>{debug_info_view()}</ul>
            })
        }
    }
}


#[component]
fn App() -> impl IntoView {
    let (content, set_content) = create_signal("**bold**".to_string());
    let (wikilinks_enabled, set_wikilinks) = create_signal(false);
    let (hard_breaks_enabled, set_hard_breaks) = create_signal(false);
    let (debug_mode, set_debug_mode) = create_signal(false);

    view!{
        <h1>Markdown editor</h1>
        <div style={"display: flex; align-items: top;"}>
            <div style="width:40%">
                <textarea type="text"
                    on:input = move |ev| set_content(event_target_value(&ev))
                    prop:value = content
                    rows={30}
                    style="margin: 10px; width: 80%"
                />
                <div>
                    <label for="wiki">enable wikilinks: </label>
                    <input type="checkbox" id="wiki"
                        on:input=move |e| set_wikilinks(event_target_checked(&e))
                    />
                </div>
                <div>
                    <label for="hardbreaks">convert soft breaks to hard breaks</label>
                    <input type="checkbox" id="hardbreaks"
                        on:input=move |e| set_hard_breaks(event_target_checked(&e))
                    />
                </div>
                <div>
                    <span>debug mode</span>
                    <input type="checkbox" name="debug-switch" 
                           on:input=move |_| set_debug_mode(true)
                    />
                </div>
            </div>
            <RenderZone content=content
                        wikilinks_enabled=wikilinks_enabled
                        hard_breaks_enabled=hard_breaks_enabled
                        debug_mode=debug_mode
            />
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App)
}
