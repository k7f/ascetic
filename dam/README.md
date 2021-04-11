ascetic_dam
===========

An asset preprocessor targeting
[maple-core](https://github.com/lukechu10/maple) and
[trunk](https://github.com/thedodd/trunk).

## Installation

Install [trunk](https://trunkrs.dev) and add these lines to
`Cargo.toml` file:

```toml
[dependencies]
maple-core = {version = "0.4", features = ["serde"]}
ascetic_dam = "0.0.1"

[build-dependencies]
ascetic_dam = "0.0.1"
```

## Usage

Suppose, the application crate's directory contains a subirectory
`assets` with a file `Assets.toml`, which is the root of a tree of
[asset-declaring manifests](#asset-declaration).  Then, the
application's `build.rs` file might look like

```rust
fn main() {
    ascetic_dam::DAM::new()
        .with_group("assets", "assets/Assets.toml")
        .with_tags(["img"])
        .with_title("Title")
        .save("index.trunk.html")
        .unwrap();

    println!("cargo:rerun-if-changed=assets");
}
```

and the file `src/main.rs` might define a module, `assets`,

```rust
#[ascetic_dam::assets(group="assets", tag="img")]
pub mod assets {}
```

decorated with attribute macro application, which populates the module
with [asset-invoking function definitions](#asset-invocation).

### Multiple root manifests and custom template

It is possible to explicitly load several manifest files,

```rust
    ascetic_dam::DAM::new()
        .with_group("icons", "assets/icons/Assets.toml")
        .with_group("badges", "assets/badges/Assets.toml")
        .with_group("styles", "assets/styles/Assets.toml")
        .with_group("scripts", "assets/scripts/Assets.toml")
        .with_tags(["img"])
        .with_template(include_str!("assets/index.tt.html"))
        .save("index.trunk.html")
        .unwrap();
```

so that definitions of asset-invoking functions may be grouped into
submodules,

```rust
pub mod assets {
    #[ascetic_dam::assets(group="icons", tag="img")]
    pub mod icons {}

    #[ascetic_dam::assets(group="badges", tag="img")]
    pub mod badges {}
}
```

There is a call to `with_template` above, which includes a custom
[tiny template](https://github.com/bheisler/TinyTemplate)
`assets/index.tt.html`, for example,

```html
<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="utf-8" />
        <title>Title</title>
        {assets | links_formatter}
    </head>
    <body>
        {assets | scripts_formatter}
    </body>
</html>
```

More verbose examples showing how to use the library may be found in
the [API documentation](https://docs.rs/ascetic_dam).

### Asset declaration

If there is a declaration

```toml
[images]
"book.svg" = { flags = ["hash"], tags = ["link", "img"], alt = "documentation" }
```

in a manifest `assets/Assets.toml`, then it refers to the file
`assets/images/book.svg` as an `<img>` element.

Assets may also be declared in non-root manifests.  Directive

```toml
["styles/Assets.toml"]
```

in `assets/Assets.toml` introduces a child manifest
`assets/styles/Assets.toml`.  The reference to a file
`assets/styles/index.scss` in the child manifest would read

```toml
["."]
"index.scss" = { tags = ["link"] }
```

### Asset invocation

```rust
template! {
    ...
    (assets::icons::book_img())
    ...
}
```

## License

`ascetic_dam` is licensed under the MIT license.  Please read the
[LICENSE-MIT](LICENSE-MIT) file in this repository for more
information.
