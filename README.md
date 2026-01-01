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