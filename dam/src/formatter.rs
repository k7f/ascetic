use tinytemplate::error::Error;

fn assets_formatter<F>(
    tag_name: &str,
    value: &serde_json::Value,
    output: &mut String,
    mut print_fn: F,
) -> Result<(), Error>
where
    F: FnMut(Option<&str>, &str, &str, &mut String),
{
    if let serde_json::Value::Object(assets) = value {
        let mut at_first = true;

        for asset in assets.values() {
            let asset = if let serde_json::Value::Object(asset) = asset {
                asset
            } else {
                return Err(Error::GenericError {
                    msg: format!("Expected a map, found {:?}.", asset),
                })
            };

            let mut has_tag = false;

            if let Some(tags) = asset.get("tags") {
                if let serde_json::Value::Array(tags) = tags {
                    for tag in tags.iter() {
                        if let serde_json::Value::String(name) = tag {
                            if name == tag_name {
                                has_tag = true;
                            }
                        } else {
                            return Err(Error::GenericError {
                                msg: format!("Expected a tag name (string), found {:?}.", tag),
                            })
                        }
                    }
                } else {
                    return Err(Error::GenericError {
                        msg: format!("Expected tags (an array), found {:?}.", tags),
                    })
                }
            } else {
                return Err(Error::GenericError { msg: "Missing tags in asset".to_string() })
            }

            if has_tag {
                let mut tag_attrs = None;

                if let Some(attrs) = asset.get("attrs") {
                    if let serde_json::Value::Object(attrs) = attrs {
                        if let Some(attrs) = attrs.get(tag_name) {
                            if let serde_json::Value::String(attrs) = attrs {
                                tag_attrs = Some(attrs);
                            } else {
                                return Err(Error::GenericError {
                                    msg: format!(
                                        "Expected attributes (string) of tag <{}>, found {:?}.",
                                        tag_name, attrs
                                    ),
                                })
                            }
                        }
                    } else {
                        return Err(Error::GenericError {
                            msg: format!("Expected attributes (a map), found {:?}.", attrs),
                        })
                    }
                }

                let work_href = if let Some(work_href) = asset.get("work_href") {
                    if let serde_json::Value::String(work_href) = work_href {
                        work_href.as_str()
                    } else {
                        return Err(Error::GenericError {
                            msg: "Invalid work_href in asset".to_string(),
                        })
                    }
                } else {
                    return Err(Error::GenericError {
                        msg: "Missing work_href in asset".to_string(),
                    })
                };

                let target_url = if let Some(target_url) = asset.get("target_url") {
                    if let serde_json::Value::String(target_url) = target_url {
                        target_url.as_str()
                    } else {
                        return Err(Error::GenericError {
                            msg: "Invalid target_url in asset".to_string(),
                        })
                    }
                } else {
                    return Err(Error::GenericError {
                        msg: "Missing target_url in asset".to_string(),
                    })
                };

                if at_first {
                    at_first = false;
                } else {
                    output.push_str("\n        ");
                }

                print_fn(tag_attrs.map(|s| s.as_str()), work_href, target_url, output);
            }
        }
        Ok(())
    } else {
        Err(Error::GenericError { msg: format!("Expected a map, found {:?}.", value) })
    }
}

pub(crate) fn links_formatter(value: &serde_json::Value, output: &mut String) -> Result<(), Error> {
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

pub(crate) fn scripts_formatter(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attrs() {
        let value = serde_json::from_str(
            r#"{ "test": {
            "tags": ["script"],
            "attrs": { "script": "defer=true" },
            "work_href": "work/test.js",
            "target_url": "test.js"}}"#,
        )
        .unwrap();
        let mut result = String::new();
        scripts_formatter(&value, &mut result).unwrap();
        assert_eq!(result.as_str(), "<script src=test.js defer=true></script>");
    }
}
