# Rust Web Markdown - Agent Guide

This repository provides markdown rendering libraries for multiple Rust web frameworks (Yew, Dioxus, Leptos). It follows a workspace structure with a core library and framework-specific implementations.

## Project Structure

This is a Cargo workspace containing:

- **`web-markdown/`** - Core library providing the abstract markdown rendering API
- **`yew-markdown/`** - Yew framework implementation  
- **`dioxus-markdown/`** - Dioxus framework implementation
- **`leptos-markdown/`** - Leptos framework implementation
- **`examples/`** - Example applications for each framework (in framework-specific directories)

## Essential Commands

### Building and Testing
```bash
# Check all workspace members compile
cargo check --all-features

# Format code (required by CI)
cargo fmt

# Build specific framework library
cargo check -p yew-markdown
cargo check -p dioxus-markdown  
cargo check -p leptos-markdown

# Build and run examples (requires Trunk)
cd yew-markdown/examples/showcase && trunk build
cd yew-markdown/examples/showcase && trunk serve
```

### WebAssembly Development
```bash
# Add WASM target if not present
rustup target add wasm32-unknown-unknown

# Build for WASM
cargo build --target wasm32-unknown-unknown
```

### Example Development with Trunk
```bash
# Install trunk if not present
cargo install trunk

# Build example for deployment
cd yew-markdown/examples/showcase
trunk build --release --public-url /rust-web-markdown/showcase

# Serve example locally for development
trunk serve
```

## Code Architecture

### Core Library Pattern
The `web-markdown` crate defines:
- `Context` trait - Abstract interface for framework-specific rendering
- `markdown_component()` function - Main entry point for parsing markdown
- `HtmlElement` enum - HTML element abstractions
- `MdComponentProps` struct - Props for custom components

### Framework Implementation Pattern
Each framework (Yew, Dioxus, Leptos) implements the `Context` trait by:
1. Creating framework-specific component props structs
2. Implementing `Context` for a reference to those props
3. Mapping `HtmlElement` enum to framework HTML elements
4. Providing a main component (e.g., `Markdown` for Yew)

### Custom Components
Each framework supports custom components via:
- `CustomComponents` struct for registration
- Component functions taking `MdComponentProps<V>` 
- Registration via `register()` method

## Code Conventions

### Rust Style
- Use `cargo fmt` for formatting (enforced by CI)
- Edition 2021
- Feature flags: `maths` (default), `debug`
- WASM compilation support required

### Framework Patterns
- Follow existing patterns when adding new framework support
- Use framework-specific HTML macros (`html!` for Yew, `view!` for Dioxus, etc.)
- Implement all `Context` trait methods
- Support the same feature flags across frameworks

### Error Handling
- Use `ComponentCreationError` for custom component errors
- Use `Result<T, ComponentCreationError>` return types
- Convert any error type to `ComponentCreationError` via `From` trait

## Testing and CI

### GitHub Actions
- **Checks workflow**: Runs `cargo check --all-features` and `cargo fmt -- --check`
- **Build workflow**: Builds examples with Trunk and deploys to GitHub Pages
- Target: `wasm32-unknown-unknown` required for all checks

### Local Testing
```bash
# Run CI checks locally
cargo check --all-features
cargo fmt -- --check

# Test examples
cd yew-markdown/examples/showcase && trunk build
```

## Dependencies

### Core Dependencies
- `pulldown-cmark` - Markdown parsing
- `syntect` - Syntax highlighting  
- `web-sys` - Web APIs
- `katex` - Math rendering (optional, feature-gated)

### Framework Dependencies
- `yew = "0.21"` for Yew implementation
- Dioxus and Leptos versions in their respective Cargo.toml files

## Gotchas and Important Notes

### WASM Development
- All code must compile for `wasm32-unknown-unknown` target
- Use conditional compilation `#[cfg(target_arch = "wasm32")]` when needed
- Trunk is required for building examples

### Feature Flags
- `maths` (default): Enables KaTeX math rendering
- `debug`: Enables debug information output
- Always test with `--all-features`

### Custom Component Registration
- Components must be registered before markdown rendering
- Use `BTreeMap` internally for component storage
- Component names are static string references

### Math Rendering
- Requires external KaTeX CSS stylesheet
- Automatically injected when `maths` feature is enabled
- Uses `MATH_STYLE_SHEET_LINK` constant for CDN resource

### Frontmatter Support
- YAML-style metadata blocks are parsed
- Accessible via `frontmatter` prop in framework components
- Use `set_frontmatter()` method in `Context` implementation

## Development Workflow

1. **Changes to core library**: Test with all framework implementations
2. **Framework-specific changes**: Only test that framework's examples  
3. **New features**: Add to core library first, then implement in frameworks
4. **Examples**: Use Trunk for development and deployment
5. **Always run**: `cargo fmt` before committing (enforced by CI)

## Adding New Framework Support

1. Create new crate in workspace with appropriate dependencies
2. Implement `Context` trait for framework's component props
3. Map all `HtmlElement` variants to framework HTML
4. Create main component following existing patterns
5. Add example application with common functionality
6. Update workspace Cargo.toml to include new member
7. Add to CI matrix if needed