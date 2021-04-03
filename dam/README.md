ascetic_dam
===========

An asset preprocessor targeting `maple-core` and `trunk`.

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

Let's assume the application crate's directory contains a subirectory
`assets` with files `index.tt.html` and `Assets.toml`, where
`index.tt.html` is a [tiny
template](https://github.com/bheisler/TinyTemplate)

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

and `Assets.toml` is the root of a tree of [asset-declaring
manifests](#asset-declaration).  Then application's `build.rs` file
might look like so:

```rust
use ascetic_dam::AssetMaker;

fn main() {
    let asset_maker = AssetMaker::new("assets/Assets.toml").unwrap();

    asset_maker.save_mod_file().unwrap();
    asset_maker.save_html_file(include_str!("assets/index.tt.html"), "index.trunk.html").unwrap();

    println!("cargo:rerun-if-changed=assets");
}
```

and `src/main.rs` might define a module `assets`

```rust
#[ascetic_dam::assets]
pub mod assets {}
```

populated with [asset-invoking function
definitions](#asset-invocation).

### Asset declaration

For example, if the root `Assets.toml` declares

```toml
[images]
"book.svg" = { flags = ["hash"], tags = ["link", "img"], alt = "documentation" }
```

then it refers to the file `assets/images/book.svg` as an `<img>`
element.  Assets may also be declared in non-root manifests.  If there
is a non-root manifest `assets/styles/Assets.toml`, then it would be
introduced in the root as

```toml
["styles/Assets.toml"]
```

and its reference to a file `assets/styles/index.scss` would read

```toml
["."]
"index.scss" = { tags = ["link"] }
```

### Asset invocation

```rust
template! {
    ...
    (assets::book_img())
    ...
}
```

## License

`ascetic_dam` is licensed under the MIT license.  Please read the
[LICENSE-MIT](LICENSE-MIT) file in this repository for more
information.
