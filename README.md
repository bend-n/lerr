# lerr

extremely barebones error diagnostics for lang-dev

![example error](https://raw.githubusercontent.com/bend-n/lerr/master/.github/example.png)

## usage

heres the code for the sample above.
feel free to add coloring with your favorite coloring crate, or just use raw ansi sequences.

```rust
use lerr::Error;
let mut e = Error::new("Strin::nouveau().i_like_tests(3.14158)");
e.label((0..5, "you probably meant String"))
    .label((7..16, "use new()"))
    .label((17..18, "caps: I"))
    .label((30..37, "your π is bad"));
eprintln!("{e}");
// dont mind this
assert_eq!(e.to_string(), "\n\u{1b}[1;34;30m0 │ \u{1b}[0mStrin::nouveau().i_like_tests(3.14158)\n\u{1b}[1;34;30m  ¦ \u{1b}[0m\u{1b}[1;34;31m──┬──\u{1b}[0m  \u{1b}[1;34;31m────┬────\u{1b}[0m \u{1b}[1;34;31m^\u{1b}[0m caps: I    \u{1b}[1;34;31m^^^^^^^\u{1b}[0m your π is bad\n\u{1b}[1;34;30m  ¦ \u{1b}[0m  \u{1b}[1;34;31m│\u{1b}[0m        \u{1b}[1;34;31m╰\u{1b}[0m use new()\n\u{1b}[1;34;30m  ¦ \u{1b}[0m  \u{1b}[1;34;31m╰\u{1b}[0m you probably meant String\n");
```

Please note that multiline labels are not yet supported.
If that doesnt work for you, use something like [ariadne](https://crates.io/crates/ariadne).