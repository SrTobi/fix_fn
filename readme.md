[![Build](https://github.com/SrTobi/fix_fn/workflows/Rust/badge.svg)](https://github.com/SrTobi/fix_fn/actions)
[![Creates.io](https://img.shields.io/crates/v/fix_fn?style)](https://crates.io/crates/fix_fn)
[![Docs](https://docs.rs/fix_fn/badge.svg)](https://docs.rs/fix_fn/)

# fix_fn

This library enables the creation of recursive closures by providing a
single macro `fix_fn`. The functionality is similar to the
[Y combinator](https://en.wikipedia.org/wiki/Fixed-point_combinator#Fixed-point_combinators_in_lambda_calculus).
Recursive closures can have arbitrary amounts of parameters and can capture
variables.

```rust
use fix_fn::fix_fn;
let fib = fix_fn!(|fib, i: u32| -> u32 {
    if i <= 1 {
           i
    } else {
        // fib will call the closure recursively
        fib(i - 1) + fib(i - 2)
    }
});

assert_eq!(fib(7), 13);
```
