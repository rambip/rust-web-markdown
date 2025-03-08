# Goal
A simple library to render markdown with dioxus, at runtime.
The best rust crates are involved!

# Usage
Add rust-web-markdown to your project:
```toml
# Cargo.toml
rust-web-markdown = {git="https://github.com/rambip/rust-web-markdown"}
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

After [setting up Dioxus](https://dioxuslabs.com/learn/0.6/guide/tooling/), in each example directory, run `dx serve --platform=web`.

You just need trunk and a web-browser to test them.

The Yew version of these examples can run in the browser from the links in [the top level ReadMe](../README.md).