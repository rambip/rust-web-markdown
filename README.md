# web-framework-markdown

This crate is a work in progress to create libraries to render markdown for the main rust web frameworks:

| Framework | Package | Crate | Source |
| --------- | ------- | ----- | ---- |
| [Dioxus](https://dioxuslabs.com/) | `dioxus-markdown` | https://crates.io/crates/dioxus-markdown | [here](./dioxus-markdown) |
| [Leptos](https://www.leptos.dev/) | `leptos-markdown` | coming soon | [here](./leptos-markdown) |
| [Yew](https://yew.rs) | `yew-markdown` | https://crates.io/crates/yew-markdown | [here](./yews-markdown) |



# Examples
Take a look at the different examples !

The following examples are build using Yew, but they are implemented for all frameworks.

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
see [here](https://rambip.github.io/rust-web-markdown/custom-components)

Custom components allow you to embed interactive or custom-styled elements in your markdown.

### Custom Component Naming Rules

To be recognized as a custom component, tag names must follow these rules:

1. **Uppercase start** - Tags starting with an uppercase letter (A-Z) are always treated as custom components
   - Examples: `<MyComponent>`, `<Counter>`, `<DataTable>`
   
2. **Lowercase with dash** - Tags starting with lowercase (a-z) must contain at least one dash (-)
   - Examples: `<my-component>`, `<data-table>`, `<custom-counter>`

These rules ensure standard HTML tags like `<div>`, `<span>`, and `<p>` are not confused with custom components.

**Valid custom components:**
- `<MyComponent>` ✓ (uppercase start)
- `<my-component>` ✓ (lowercase start with dash)
- `<Counter initial="5"/>` ✓ (uppercase, self-closing with attributes)

**NOT custom components:**
- `<div>` ✗ (lowercase without dash - standard HTML)
- `<span>` ✗ (lowercase without dash - standard HTML)
- `<p>` ✗ (lowercase without dash - standard HTML)

# Contribute

PRs are **very much** appreciated.

# Changelog

## 0.1.0

- Mark `ElementAttributes` as `non_exhaustive` so future changes can add more attributes as a non breaking change.
- Include crate README.
- Require use of methods to access contents of `MdComponentProps`'s attributes.
    This also removes the ability fore consumers to directly construct a  `MdComponentProps` instance.
- Expose the location of `MdComponentProps` attributes in the source as a range.
- `HtmlError` has been removed from the package API.
- Replace `katex`with `katex-rs` for math rendering, improving performance and platform support.
- Fix empty crash on empty codeblocks.
- Add opt out for preserving html.
- Documentation improvements.