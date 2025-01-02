# Goal
A simple library to render markdown with dioxus, at runtime.
The best rust crates are involved !

# Usage
Add dioxus-markdown to your project:
```toml
# Cargo.toml
dioxus-markdown = {git="https://github.com/rambip/dioxus-markdown"}
```

If you just need to render basic markdown, you can do

```rust
use dioxus_markdown::Markdown;
...
    rsx!{
        Markdown {src:"# Mardown power !"}
    }
```

# Examples
Take a look at the different examples !
You just need trunk and a web-browser to test them.

## Showcase
the example is included in `./examples/showcase`

Here is an illustration:
![](./img/showcase.jpg)

see [here](https://rambip.github.io/dioxus-markdown/showcase)

## Editor
Of course, a basic markdown editor is included.

You can test for yourself [here](https://rambip.github.io/dioxus-markdown/editor) !

## Interactivity
see [here](https://rambip.github.io/dioxus-markdown/onclick)

## Custom Components

You can define your own components in your code and call them inside markdown !

see [here](https://rambip.github.io/dioxus-markdown/custom_components)
