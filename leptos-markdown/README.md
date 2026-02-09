# `leptos-markdown`

A zero-config but extendable markdown component for [leptos](https://www.leptos.dev/).

It supports [commonmark](https://commonmark.org/), and so much more.

# Installation
`leptos-markdown` is not published on crates.io yet.
Use it as a git dependency !
```toml
# inside Cargo.toml
leptos-markdown = {git="https://github.com/rambip/rust-web-markdown"}
```

# Usage
You can use this component to render both static and dynamic markdown.

## Static markdown

```rust
use leptos::*;
use leptos_markdown::Markdown;

{
    ...
    view!{
        <Markdown src="# Markdown Power !"/>
    }
}
```

## Dynamic markdown
```rust
{
    ...
    let (content, set_content) = create_signal("# Markdown Power !".to_string());

    view!{
        <Markdown src=content/>
    }
}
```


# Examples
To build them, just follow the [leptos installation instructions](https://leptos-rs.github.io/leptos/02_getting_started.html) and run `trunk serve` to try them.

## Showcase
![](./showcase.jpg)

`./examples/showcase`

You can see the result [here](https://rambip.github.io/rust-web-markdown/showcase)

To be fair, this is not the vanilla component, there is a bit of styling added.

## Editor
`./examples/editor`

There is a demo of an interactive editor [here](https://rambip.github.io/rust-web-markdown/editor)

## Onclick
`./examples/onclick/`

Illustrates a cool feature of this crate: `onclick` events for any rendered content

Try it [here](https://rambip.github.io/rust-web-markdown-markdown/onclick)

## Custom components

This feature is still very experimental.
But there is an example [here](https://rambip.github.io/rust-web-markdown-markdown/custom_component)

Custom components allow you to embed interactive Leptos components in your markdown.

### Custom Component Naming Rules

To be recognized as a custom component, tag names must follow these rules:

1. **Uppercase start** - Tags starting with an uppercase letter (A-Z) are always treated as custom components
   - Examples: `<MyComponent>`, `<Counter>`, `<DataTable>`
   
2. **Lowercase with dash** - Tags starting with lowercase (a-z) must contain at least one dash (-)
   - Examples: `<my-component>`, `<data-table>`, `<custom-counter>`

These rules ensure standard HTML tags like `<div>`, `<span>`, and `<p>` are not confused with custom components.

See the [custom-component example](./examples/custom-component) for a complete working example.

# Changelog

## 0.7.0

 - Update `web-framework-markdown` to `0.1`.
