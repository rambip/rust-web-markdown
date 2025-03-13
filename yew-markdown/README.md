# Goal
Creating a simple library to render markdown with yew.
The best rust crates are involved !

# Usage
Add yew-markdown to your project:
```toml
# Cargo.toml
yew-markdown = "0.0.1"
```

If you just need to render basic markdown, you can do

```rust
use yew_markdown::Markdown;
...
    html!{
        <Markdown src={"# Markdown power !"}/>
    }
```

# Examples
Take a look at the different examples !
You just need trunk and a web-browser to test them.

## Showcase
the example is included in `./examples/showcase`

Here is an illustration:
![](./img/showcase.jpg)

see [here](https://rambip.github.io/rust-web-markdown/showcase)

## Editor
Of course, an example of a basic markdown editor is implemented to show what is currently supported

see [here](https://rambip.github.io/rust-web-markdown/editor)

## Interactivity
see [here](https://rambip.github.io/rust-web-markdown/onclick)

## Custom Components
see [here](https://rambip.github.io/rust-web-markdown/custom_components)
