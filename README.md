# lerr

extremely barebones error diagnostics for lang-dev

![example error](https://raw.githubusercontent.com/bend-n/lerr/master/.github/example.png)

## usage

heres the code for the sample above.
coloring is done with the [comat](https://docs.rs/comat) crate.

```rust
use comat::cformat as cmt;
use lerr::Error;
let mut e = Error::new("Strin::nouveau().i_like_tests(3.14158)");
e.message(cmt!(r#"{bold_red}error{reset}: unknown function {bold_red}String::new(){reset}"#))
    .label((0..5, cmt!("you probably meant {black}String{reset}")))
    .label((7..16, cmt!("use {green}new(){reset}")))
    .label((17..18, cmt!("caps: {bold_cyan}I{reset}")))
    .label((30..37, cmt!("your {bold_yellow}π{reset} is bad")));
eprintln!("{e}");
```

Please note that multiline labels are not yet supported.
If that doesnt work for you, use something like [ariadne](https://crates.io/crates/ariadne).
