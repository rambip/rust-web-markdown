This project was great to compare the advantages and drawbacks of the two major rust web frameworks !

I plan to write a post to explain that in detail, but if you are curious they are some things I already noticed:

## Leptos is better for
- `async` with `Suspense` ! The idea of `create_resource` is really intuitive and great ! Though, it has bugs (see below)
- performance: it is both faster and more memory-efficient than Yew; since my App is about 2Mo, it is nice
- Abstraction: the fact that all your context is inside `cx`, which is `Copy`, makes everything simpler !!! 
- Ergonomics to access the js world: you can do an input form in 3 lines with `on:input` and `event_target_into`. In yew you needed a whole file to define your input element !
- Styling: Leptos has default styling solutions, while Yew doesn't. It's always better to have a standard, even if it is not perfect

## But ...
- the documentation is not as clear as the `Yew` one. The content is quite good, but it's hader to navigate IMO. I found more answers in the discord than in the docs ...
- Really Buggy: I opened 2 issues (like [this one](https://github.com/leptos-rs/leptos/issues/1456)) in one single day
- `Box<dyn Fn(A) -> ...>` are a pain to work with, and it is the recommended way to work with generic optional parameters
- Closures, closures, closures, closures !!! If you are not 100% confident about how closures in Rust work, you will get stuck at one point.

## Small conclusion
Leptos is really promising, and I think it could soon become a great choice for easy and safe web-developpment. In particular it's concise, has zero boilerplate and is very flexible.
But I can't say I really enjoyed working with it for now: too many bugs, some things missing in the documentation, and I miss an intermediate guide on closure to survive the Leptos world 😟

Within one year, it can become the greatest rust web framework. Maybe.
So go over to https://leptos.dev/ and see if you like it !
