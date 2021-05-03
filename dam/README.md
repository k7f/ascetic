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
    ascetic_dam::Collection::new()
        .with_group("assets", "assets/Assets.toml")
        .with_tags(["img"])
        .with_title("Title")
        .render("index.trunk.html")
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
ascetic_dam::Collection::new()
    .with_group("styles", "assets/styles/Assets.toml")
    .with_group("scripts", "assets/scripts/Assets.toml")
    .with_group("icons", "assets/icons/Assets.toml")
    .with_group("badges", "assets/badges/Assets.toml")
    .with_tags(["img"])
    .with_html_template(include_str!("assets/index.tt.html"))
    .render("index.trunk.html")
    .unwrap();
}
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

The call to `with_html_template` above includes a custom [tiny
template](https://github.com/bheisler/TinyTemplate)
`assets/index.tt.html`.  This call is optional &mdash; without it, the
default template,

```html
<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="utf-8" />
        <title>{title}</title>
        {assets | link_assets_formatter}
    </head>
    <body>
        {elements | elements_formatter}
        {assets | script_assets_formatter}
    </body>
</html>
```

is used.  The path to the rendered HTML file is to be given in the
call to `Collection::render`.

Also optionally, in order to refer to assets in style sheets, one may
call `with_scss_template`, which takes two arguments: source of a
style template, and a path to where a rendered SCSS file is to be
output.  Since, unlike the HTML case, there is no default style
template, at least one call to `with_scss_template` is required for
style sheet rendering to take place at all.  Two formatters available
in style templates are

```scss
// generate `@import` directives
{assets | import_assets_formatter}

// generate `@extend`ing class definitions
{assets | extend_assets_formatter}
```

A successful call to `Collection::render` will always trigger
rendering of exactly one HTML template (explicit or default), and of
zero, one, or more style templates.

Additionally, a HTML template may trigger rendering of the array of
`elements` (see the default template above).  This array is populated
by calls to `Collection::with_element`, such as the one in the
following `build.rs` script.

```rust
use ascetic_dam::{Collection, AssetError};

fn collect_assets() -> Result<Collection, AssetError> {
    Collection::new()
        .with_group("styles", "assets/styles/Assets.toml")
        .with_group("scripts", "assets/scripts/Assets.toml")
        .with_group("icons", "assets/icons/Assets.toml")
        .with_group("badges", "assets/badges/Assets.toml")
        .with_tags(["img"])
        .with_html_template(include_str!("assets/index.tt.html"))
        .with_element("div", "id=\"root\"")?
        .with_scss_template(include_str!("assets/index.tt.scss"), "index.scss")
}

fn main() {
    collect_assets().unwrap().render("index.trunk.html").unwrap();
    println!("cargo:rerun-if-changed=assets");
}
```

More verbose examples showing how to use the library may be found in
the [API documentation](https://docs.rs/ascetic_dam).

### Asset declaration

If there is a declaration

```toml
[images]
"book.svg" = { flags = ["hash"], tags = ["link", "img"], extends = ["btn-icon"], alt = "documentation" }
```

in a manifest `assets/Assets.toml`, then it refers to the file
`assets/images/book.svg` as an `<img>` element, and as a
`background-image` property of the `.btn-icon-book` SCSS class.

Assets may also be declared in non-root manifests.  A
directive

```toml
["styles/Assets.toml"]
```

if included in the manifest `assets/Assets.toml`, introduces a child
manifest `assets/styles/Assets.toml`.  The reference to a file
`assets/styles/index.scss` in the child manifest would then read

```toml
["."]
"index.scss" = { tags = ["link"] }
```

if the style sheet is to be `<link>`ed in the rendered HTML file, or

```toml
["."]
"index.scss" = { flags = ["import"] }
```

if it is to be `@import`ed in a parent style sheet declared in the
call to `with_scss_template`.

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
