# dioxus-markdown
A simple library to render markdown with Dioxus, at runtime.
The best rust crates are involved!

# Usage
Add dioxus-markdown to your project:
```toml
# Cargo.toml
dioxus-markdown = "0.1.0"
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
Take a look at the different examples!

After [setting up Dioxus](https://dioxuslabs.com/learn/0.7/tutorial/tooling), in each example directory, run `dx serve --platform=web`.

You just need trunk and a web-browser to test them.

The Yew version of these examples can run in the browser from the links in [the top level ReadMe](../README.md).

## Custom Components

Custom components allow you to embed interactive Dioxus components in your markdown.

### Custom Component Naming Rules

To be recognized as a custom component, tag names must follow these rules:

1. **Uppercase start** - Tags starting with an uppercase letter (A-Z) are always treated as custom components
   - Examples: `<MyComponent>`, `<Counter>`, `<DataTable>`
   
2. **Lowercase with dash** - Tags starting with lowercase (a-z) must contain at least one dash (-)
   - Examples: `<my-component>`, `<data-table>`, `<custom-counter>`

These rules ensure standard HTML tags like `<div>`, `<span>`, and `<p>` are not confused with custom components.

See the [custom-components example](./examples/custom-components) for a complete working example.

# Changelog

## 0.1.0

 - Update `web-framework-markdown` to `0.1`.
 - Update Dioxus to `0.7`.
 - Include experimental `substring::ReadWriteBox` to help with custom components which can modify their attributes.
 