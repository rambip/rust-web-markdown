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

Custom components allow you to embed interactive Yew components in your markdown.

### Custom Component Naming Rules

To be recognized as a custom component, tag names must follow these rules:

1. **Uppercase start** - Tags starting with an uppercase letter (A-Z) are always treated as custom components
   - Examples: `<MyComponent>`, `<Counter>`, `<DataTable>`
   
2. **Lowercase with dash** - Tags starting with lowercase (a-z) must contain at least one dash (-)
   - Examples: `<my-component>`, `<data-table>`, `<custom-counter>`

These rules ensure standard HTML tags like `<div>`, `<span>`, and `<p>` are not confused with custom components.

See the [custom-components example](./examples/custom-components) for a complete working example.

