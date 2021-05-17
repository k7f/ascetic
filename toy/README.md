ascetic_toy
===========
[![Latest version](https://img.shields.io/crates/v/ascetic_toy.svg)](https://crates.io/crates/ascetic_toy)
[![docs](https://docs.rs/ascetic_toy/badge.svg)](https://docs.rs/ascetic_toy)
![Rust](https://img.shields.io/badge/rust-nightly-brightgreen.svg)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

Analysis and synthesis of [cause-effect
synchronised](https://link.springer.com/book/10.1007/978-3-030-20461-7)
interacting systems.  This is a set of command-line tools of the
[_Ascesis_](https://github.com/k7f/ascesis) project.

## Prerequisites

In principle, `ascetic_toy` should build wherever `rustc` and `cargo`
runs.  Its executables should run on any
[platform](https://forge.rust-lang.org/release/platform-support.html)
supporting the Rust `std` library.  Be aware, though, that the project
is very much a WIP.  The main toolchain used in development is nightly
channel of Rust 1.45.

## Installation

Having [Rust](https://www.rust-lang.org/downloads.html) installed,
ensure its version is at least 1.45: check with `cargo version` and
run `rustup update` if needed.  Then

```bash
$ cargo install ascetic_toy
```

will automatically download, build, and install the latest
`ascetic_toy` release on
[crates.io](https://crates.io/crates/ascetic_toy).

## Command line interface

C-e structures may be defined in text files by using the
[_Ascesis_](https://github.com/k7f/ascesis) language or the `.cex`
format of textual description.  The _Ascesis_ language has formally
specified
[syntax](https://github.com/k7f/ascesis/blob/master/spec/ascesis-syntax.ebnf)
and informally described
[semantics](https://github.com/k7f/ascesis/blob/master/spec/parser-implementation.md).
The format of `.cex` text files is YAML-based, but nowhere documented
and very likely to change (there are some, perhaps self-explanatory,
[examples](../scripts/cex)).

Run the `ascesis` executable to load c-e structures from (one or more)
`.ces` or `.cex` files and analyse them.  By default, the program will
check link coherence and print firing components, if there are any, or
inform about structural deadlock.  When instructed, it may execute one
step of a simulation,

```bash
$ ascesis scripts/examples/zerotest.ces -f entry tested
```

run a longer simulation,

```bash
$ ascesis scripts/examples/arrow.ces scripts/examples/choice-two.ces -f a:3 -x 3
```

or validate a set of files,

```bash
$ ascesis validate -r scripts
```

To see the list of available subcommands and options run

```bash
$ ascesis --help
```

## License

`ascetic_toy` is licensed under the MIT license.  Please read the
[LICENSE-MIT](LICENSE-MIT) file in this repository for more
information.
