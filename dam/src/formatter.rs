use tinytemplate::error::Error;

fn assets_formatter<F1, F2>(
    value: &serde_json::Value,
    output: &mut String,
    filter_fn: F1,
    mut print_fn: F2,
) -> Result<(), Error>
where
    F1: Fn(&[serde_json::Value]) -> bool,
    F2: FnMut(&str, &str, &mut String),
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

            let tags = if let Some(tags) = asset.get("tags") {
                if let serde_json::Value::Array(tags) = tags {
                    tags.as_slice()
                } else {
                    return Err(Error::GenericError {
                        msg: format!("Expected an array, found {:?}.", tags),
                    })
                }
            } else {
                return Err(Error::GenericError { msg: "Missing tags in asset".to_string() })
            };

            if filter_fn(tags) {
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

                print_fn(work_href, target_url, output);
            }
        }
        Ok(())
    } else {
        Err(Error::GenericError { msg: format!("Expected a map, found {:?}.", value) })
    }
}

pub(crate) fn links_formatter(value: &serde_json::Value, output: &mut String) -> Result<(), Error> {
    assets_formatter(
        value,
        output,
        |tags| {
            tags.iter().any(
                |v| {
                    if let serde_json::Value::String(v) = v {
                        v == "link"
                    } else {
                        false
                    }
                },
            )
        },
        |work_href, _target_url, output| {
            if work_href.ends_with(".scss") {
                output.push_str(&format!("<link data-trunk rel=\"scss\" href={} />", work_href));
            } else if work_href.ends_with(".css") {
                output.push_str(&format!("<link data-trunk rel=\"css\" href={} />", work_href));
            } else {
                output
                    .push_str(&format!("<link data-trunk rel=\"copy-file\" href={} />", work_href));
            }
        },
    )
}

pub(crate) fn scripts_formatter(value: &serde_json::Value, output: &mut String) -> Result<(), Error> {
    assets_formatter(
        value,
        output,
        |tags| {
            tags.iter().any(|v| {
                if let serde_json::Value::String(v) = v {
                    v == "script"
                } else {
                    false
                }
            })
        },
        |_work_href, target_url, output| {
            output.push_str(&format!("<script src={}></script>", target_url));
        },
    )
}
