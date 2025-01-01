use leptos::*;
use leptos_markdown::Markdown;

static MARKDOWN_SOURCE: &str = r#"
## Code
```rust
fn main() {
    println!("hello world !")
}
```

## Math
- $1+1=2$

- $e^{i\pi}+1=0$


$$\int_0^{+\infty}\dfrac{\sin(t)}{t}\,dt=\dfrac{\sqrt{\pi}}{2}$$


## Links and images
![](https://raw.githubusercontent.com/wooorm/markdown-rs/8924580/media/logo-monochromatic.svg?sanitize=true)

for markdown documentation, see [here](https://commonmark.org/help/)

Wikilinks are supported too: [[https://en.wikipedia.org/wiki/Markdown|markdown]]

## Style
| unstyled | styled    |
| :-----:  | ------    |
| bold     | **bold**  |
| italics  | *italics* |
| strike   | ~strike~  |

> Hey, I am a quote !

## Lists
1) one
2) two
3) three

- and
- unorderded
- too

Even todo lists:
- [ ] todo
- [x] done
"#;

#[component]
fn App() -> impl IntoView {
    view!{
        <Markdown src=MARKDOWN_SOURCE wikilinks=true/>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App)
}
