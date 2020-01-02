Ascetic
=======
[![Latest version](https://img.shields.io/crates/v/ascetic_cli.svg)](https://crates.io/crates/ascetic_cli)
[![docs](https://docs.rs/ascetic_cli/badge.svg)](https://docs.rs/ascetic_cli)
![Rust](https://img.shields.io/badge/rust-nightly-brightgreen.svg)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

Analysis and synthesis of [cause-effect
synchronised](https://link.springer.com/book/10.1007/978-3-030-20461-7)
interacting systems.  This is a set of command-line tools of the
[_Ascesis_](https://github.com/k7f/ascesis) project.

## Prerequisites

In principle, `ascetic_cli` should build wherever `rustc` and `cargo`
runs.  Its executables should run on any
[platform](https://forge.rust-lang.org/platform-support.html)
supporting the Rust `std` library.  Be aware, though, that the project
is very much a WIP.  The main toolchain used in development is nightly
channel of Rust 1.42.

## Installation

Having [Rust](https://www.rust-lang.org/downloads.html) installed,
ensure its version is at least 1.42: check with `cargo version` and
run `rustup update` if needed.  Then

```bash
$ cargo install ascetic_cli
```

will automatically download, build, and install the latest
`ascetic_cli` release on
[crates.io](https://crates.io/crates/ascetic_cli).

## Command line interface

C-e structures may be defined in `.cex` text files.  The format of
textual description is YAML-based, but nowhere documented and very
likely to change.  There are some, perhaps self-explanatory,
[examples](data/).

Run the `ascesis` executable to load c-e structures from `.cex` files
and analyse them.  By default, the program will check link coherence
and print firing components, if there are any, or inform about
structural deadlock.  To see the list of available subcommands and
options run

```bash
$ ascesis --help
```

## License

`ascetic_cli` is licensed under the MIT license.  Please read the
[LICENSE-MIT](LICENSE-MIT) file in this repository for more
information.
