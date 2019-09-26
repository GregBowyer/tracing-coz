# tracing-coz

[Rust-tracing](https://github.com/tokio-rs/tracing) support for the [`coz`
Causal Profiler](https://github.com/plasma-umass/coz)

[![Documentation](https://docs.rs/tracing-coz/badge.svg)](https://docs.rs/tracing-coz/)

## Usage

First, follow the instructions in [`coz`] to install the `coz` command.

[`coz`]: https://github.com/plasma-umass/coz/#installation

Note that this crate is a facade over
[rust-coz](https://github.com/alexcrichton/coz-rs) as such its information
applies to this crate.

With this crate `spans` are mapped to `coz::begin!` and `coz::end!`, and
`events` are mapped to `coz::progress!`, as throughput or latency tracepoints
respectively. More information on this [can be found
upstream](https://github.com/plasma-umass/coz/#profiling-modes). 

After you've instrumented your code, you need to also ensure that you're
compiling with DWARF debug information. To do this you'll want to configure
`Cargo.toml` again:

```toml
[profile.release]
debug = 1
```

Next up you'll build your application with `cargo build --release`, and then
finally you can run it with `coz run --- ./target/release/$your_binary`.

## Caveats

Known caveats so far to generate a report that collects information are:

* Rust programs by default segfault when run with `coz` with an issue related to
  [plasma-umass/coz#110](https://github.com/plasma-umass/coz/issues/110). Rust
  programs set up a `sigaltstack` to run segfault handlers to print "you ran out
  of stack", but this alternate stack is too small to run the `SIGPROF` handler
  that `coz` installs. To handle this this crate provides a `coz::thread_init()`
  function which will increase the `sigaltstack` size that Rust installs by
  default to something large enough to run `coz`. If you see segfaults, or
  corrupt reports, you may wish to manually call `coz::thread_init()` instead of
  waiting for this crate to automatically call it for you, we export
  `coz::thread_init()` for convenience here.

* Debug information looks to be critical to get a report from `coz`. Make sure
  that your program is compiled with at least line-table information (`debug =
  1`) to ensure you get the best experience using `coz`.

* Currently `coz` only works on Linux, and while this crate should compile on
  all platforms it only actually does something on Linux.

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
