# lerr

extremely barebones error diagnostics for lang-dev

![example error](https://raw.githubusercontent.com/bend-n/lerr/master/.github/example.png)

## usage

heres the code for the sample above.
feel free to add coloring with your favorite coloring crate, or just use raw ansi sequences.

```rust
use lerr::Error;
let out = Error::new("Strin::new()")
    .label((0..5, "a 'strin' you say"))
    .note("i think you meant String")
    .to_string();
println!("{out}");
```

Please note that only one label per line is currently supported, and multiline labels are not yet supported.
If that doesnt work for you, use something like [ariadne](https://crates.io/crates/ariadne).