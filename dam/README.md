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
might look like

```rust
use ascetic_dam::{AssetGroup, AssetMaker};

fn main() {
    let asset_group = AssetGroup::new("assets", "assets/Assets.toml").unwrap();

    asset_group.save_mod_files(&["img"]).unwrap();
    asset_group.save_html_file(include_str!("assets/index.tt.html"), "index.trunk.html").unwrap();

    println!("cargo:rerun-if-changed=assets");
}
```

or

```rust
use ascetic_dam::{AssetGroup, AssetMaker};

fn main() {
    let icon_group = AssetGroup::new("icons", "assets/icons/Assets.toml").unwrap();
    let badge_group = AssetGroup::new("badges", "assets/badges/Assets.toml").unwrap();
    let style_group = AssetGroup::new("styles", "assets/styles/Assets.toml").unwrap();
    let script_group = AssetGroup::new("scripts", "assets/scripts/Assets.toml").unwrap();

    icon_group.save_mod_files(&["img"]).unwrap();
    badge_group.save_mod_files(&["img"]).unwrap();

    [icon_group, badge_group, style_group, script_group]
        .save_html_file(include_str!("assets/index.tt.html"), "index.trunk.html")
        .unwrap();

    println!("cargo:rerun-if-changed=assets");
}
```

and `src/main.rs` might define a module, `assets`,

```rust
pub mod assets {
    #[ascetic_dam::assets(group="icons", tag="img")]
    pub mod icons {}

    #[ascetic_dam::assets(group="badges", tag="img")]
    pub mod badges {}
}
```

containing submodules populated with [asset-invoking function
definitions](#asset-invocation).

### Asset declaration

If the root manifest declares, for example,

```toml
[images]
"book.svg" = { flags = ["hash"], tags = ["link", "img"], alt = "documentation" }
```

then it refers to the file `assets/images/book.svg` as an `<img>`
element.

Assets may also be declared in non-root manifests.  If there is a
non-root manifest `assets/styles/Assets.toml`, then it would be
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
    (assets::icons::book_img())
    ...
}
```

## License

`ascetic_dam` is licensed under the MIT license.  Please read the
[LICENSE-MIT](LICENSE-MIT) file in this repository for more
information.
