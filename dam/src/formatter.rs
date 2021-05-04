use tinytemplate::error::Error;

#[inline]
fn get_string<'a>(value: &'a serde_json::Value, name: &str) -> Result<&'a String, Error> {
    if let serde_json::Value::String(result) = value {
        Ok(result)
    } else {
        Err(Error::GenericError {
            msg: format!("Expected {} (a string), found {:?}.", name, value),
        })
    }
}

#[inline]
fn get_array<'a>(
    value: &'a serde_json::Value,
    name: &str,
) -> Result<&'a Vec<serde_json::Value>, Error> {
    if let serde_json::Value::Array(result) = value {
        Ok(result)
    } else {
        Err(Error::GenericError {
            msg: format!("Expected {} (an array), found {:?}.", name, value),
        })
    }
}

#[inline]
fn get_object<'a>(
    value: &'a serde_json::Value,
    name: &str,
) -> Result<&'a serde_json::Map<String, serde_json::Value>, Error> {
    if let serde_json::Value::Object(result) = value {
        Ok(result)
    } else {
        Err(Error::GenericError { msg: format!("Expected {} (a map), found {:?}.", name, value) })
    }
}

#[inline]
fn get_value_for_key<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    key: &str,
    object_name: &str,
) -> Result<&'a serde_json::Value, Error> {
    if let Some(result) = object.get(key) {
        Ok(result)
    } else {
        Err(Error::GenericError { msg: format!("Missing `{}` in {}.", key, object_name) })
    }
}

fn assets_formatter<F>(
    tag_name: &str,
    value: &serde_json::Value,
    output: &mut String,
    mut print_fn: F,
) -> Result<(), Error>
where
    F: FnMut(Option<&str>, &str, &str, &mut String),
{
    let assets = get_object(value, "assets")?;
    let mut at_first = true;

    for asset in assets.values() {
        let mut has_tag = false;
        let asset = get_object(asset, "asset")?;
        let tags = get_value_for_key(asset, "tags", "asset")?;
        let tags = get_array(tags, "tags")?;

        for tag in tags.iter() {
            if get_string(tag, "tag name")? == tag_name {
                has_tag = true;
                // parse through for consistent error detection
            }
        }

        if has_tag {
            let mut tag_attrs = None;

            if let Some(attrs) = asset.get("attrs") {
                let attrs = get_object(attrs, "attributes")?;

                if let Some(attrs) = attrs.get(tag_name) {
                    let attrs =
                        get_string(attrs, format!("attributes of tag <{}>", tag_name).as_str())?;

                    tag_attrs = Some(attrs);
                }
            }

            let work_href = get_value_for_key(asset, "work_href", "asset")?;
            let work_href = get_string(work_href, "work_href of asset")?;
            let target_url = get_value_for_key(asset, "target_url", "asset")?;
            let target_url = get_string(target_url, "target_url of asset")?;

            if at_first {
                at_first = false;
            } else {
                output.push_str("\n        ");
            }

            print_fn(tag_attrs.map(|s| s.as_str()), work_href, target_url, output);
        }
    }

    Ok(())
}

pub(crate) fn link_assets_formatter(
    value: &serde_json::Value,
    output: &mut String,
) -> Result<(), Error> {
    assets_formatter("link", value, output, |_attrs, work_href, _target_url, output| {
        if work_href.ends_with(".scss") {
            output.push_str(&format!("<link data-trunk rel=\"scss\" href={} />", work_href));
        } else if work_href.ends_with(".css") {
            output.push_str(&format!("<link data-trunk rel=\"css\" href={} />", work_href));
        } else {
            output.push_str(&format!("<link data-trunk rel=\"copy-file\" href={} />", work_href));
        }
    })
}

pub(crate) fn script_assets_formatter(
    value: &serde_json::Value,
    output: &mut String,
) -> Result<(), Error> {
    assets_formatter("script", value, output, |attrs, _work_href, target_url, output| {
        if let Some(attrs) = attrs {
            output.push_str(&format!("<script src={} {}></script>", target_url, attrs));
        } else {
            output.push_str(&format!("<script src={}></script>", target_url));
        }
    })
}

pub(crate) fn elements_formatter(
    value: &serde_json::Value,
    output: &mut String,
) -> Result<(), Error> {
    let elements = get_array(value, "elements")?;

    for element in elements {
        let element = get_object(element, "element")?;
        let tags = get_value_for_key(element, "tags", "element")?;
        let tags = get_array(tags, "tags")?;

        for tag_name in tags {
            let tag_name = get_string(tag_name, "tag name")?;

            if let Some(attrs) = element.get("attrs") {
                let attrs = get_object(attrs, "attrs")?;

                if let Some(attrs) = attrs.get(tag_name) {
                    let attrs =
                        get_string(attrs, format!("attributes of tag <{}>", tag_name).as_str())?;

                    output.push_str(&format!("<{0} {1}></{0}>", tag_name, attrs));
                    continue
                }
            }
            output.push_str(&format!("<{0}></{0}>", tag_name));
        }
    }

    Ok(())
}

pub(crate) fn import_assets_formatter(
    value: &serde_json::Value,
    output: &mut String,
) -> Result<(), Error> {
    let assets = get_object(value, "assets")?;

    for asset in assets.values() {
        let asset = get_object(asset, "asset")?;
        let flags = get_value_for_key(asset, "flags", "asset")?;
        let flags = get_array(flags, "flags")?;

        for flag in flags.iter() {
            let flag = get_string(flag, "flag")?;

            if flag == "import" {
                let source_path = get_value_for_key(asset, "source_path", "asset")?;
                let source_path = get_string(source_path, "source path")?;
                let path = std::env::current_dir()
                    .map_err(|_| Error::GenericError { msg: "current dir is unknown".to_string() })?
                    .join(source_path);

                output.push_str(&format!("@import {:?};", path));
                break
            }
        }
    }

    Ok(())
}

pub(crate) fn extend_assets_formatter(
    value: &serde_json::Value,
    output: &mut String,
) -> Result<(), Error> {
    let assets = get_object(value, "assets")?;

    for (key, asset) in assets {
        let asset = get_object(asset, "asset")?;

        if let Some(extends) = asset.get("extends") {
            let extends = get_array(extends, "extends")?;

            for class_name in extends {
                let class_name = get_string(class_name, "css class name")?;
                let suffix =
                    if let Some((_prefix, suffix)) = key.rsplit_once("::") { suffix } else { key };
                let suffix: String =
                    suffix.chars().map(|c| if c == '_' { '-' } else { c }).collect();
                let target_url = get_value_for_key(asset, "target_url", "asset")?;
                let target_url = get_string(target_url, "target URL")?;

                output.push_str(&format!(
                    "\n.{}-{} {{
    @extend .{};
    background-image: url('{}');
}}\n",
                    class_name, suffix, class_name, target_url
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attrs_from_toml() {
        let decl = toml::from_str(
            r#"
            tags = ["script"]
            attrs = { "script" = "defer=true" }
            "#,
        )
        .expect("toml parsing error");
        let mut group = crate::AssetGroup::default();
        let (key, asset) = group.create_asset("test.js", ".", decl).expect("asset creation error");
        group.register_asset(key, asset).unwrap();
        let mut tt = tinytemplate::TinyTemplate::new();
        tt.add_formatter("script_assets_formatter", script_assets_formatter);
        tt.add_template("html", r#"{assets | script_assets_formatter}"#).expect("bad template");
        let result = tt.render("html", &group).expect("template rendering error");
        assert_eq!(result.as_str(), "<script src=test.js defer=true></script>");
    }

    #[test]
    fn attrs_from_json() {
        let value = serde_json::from_str(
            r#"{ "test": {
            "tags": ["script"],
            "attrs": { "script": "defer=true" },
            "work_href": "work/test.js",
            "target_url": "test.js"}}"#,
        )
        .expect("json parsing error");
        let mut result = String::new();
        script_assets_formatter(&value, &mut result).expect("script assets formatter error");
        assert_eq!(result.as_str(), "<script src=test.js defer=true></script>");
    }

    #[test]
    fn scss_from_toml() {
        let scss_decl = toml::from_str(
            r#"
            flags = ["import"]
            "#,
        )
        .expect("toml parsing error");
        let svg_decl = toml::from_str(
            r#"
            extends = ["base-class"]
            "#,
        )
        .expect("toml parsing error");
        let mut group = crate::AssetGroup::default();
        let (key, asset) =
            group.create_asset("style.scss", ".", scss_decl).expect("asset creation error");
        group.register_asset(key, asset).expect("asset registration error");
        let (key, asset) =
            group.create_asset("icon.svg", ".", svg_decl).expect("asset creation error");
        group.register_asset(key, asset).expect("asset registration error");
        let mut tt = tinytemplate::TinyTemplate::new();
        tt.add_formatter("import_assets_formatter", import_assets_formatter);
        tt.add_formatter("extend_assets_formatter", extend_assets_formatter);
        tt.add_template(
            "scss",
            r#"{assets | import_assets_formatter}
            {assets | extend_assets_formatter}"#,
        )
        .expect("bad template");
        let result = tt.render("scss", &group).expect("template rendering error");
        assert!(result.starts_with("@import \""));
        assert!(result.ends_with(
            ".base-class-icon {\n    @extend .base-class;\n    background-image: \
             url('icon.svg');\n}\n"
        ));
    }
}
