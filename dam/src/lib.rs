use std::{
    path::{Path, PathBuf},
    collections::{HashMap, HashSet},
    io::Write,
};
use tracing::{warn, error};
use serde::{Serialize, Deserialize};
use tinytemplate::{TinyTemplate, error::Error as TTError};

pub use ascetic_dam_macro::assets;

#[derive(Serialize, Clone, Debug)]
pub struct Asset {
    source_path: PathBuf,
    work_href:   String,
    target_url:  String,
    tags:        Vec<String>,
    #[serde(skip_serializing)]
    decl:        AssetDeclaration,
}

impl Asset {
    pub fn as_html_element<S: AsRef<str>>(&self, tag: S) -> Result<String, std::io::Error> {
        let tag = tag.as_ref();
        match tag {
            "img" => Ok(self.as_img()),
            _ => {
                let msg = format!("HTML element with tag `{}` isn't supported", tag);
                Err(std::io::Error::new(std::io::ErrorKind::Other, msg))
            }
        }
    }

    pub fn as_img(&self) -> String {
        if let Some(ref alt) = self.decl.alt {
            format!(
                "img(src=\"{}\", width=\"{}\", height=\"{}\", alt=\"{}\")",
                self.target_url,
                self.decl.width.unwrap_or(32),
                self.decl.height.unwrap_or(32),
                alt,
            )
        } else {
            format!(
                "img(src=\"{}\", width=\"{}\", height=\"{}\")",
                self.target_url,
                self.decl.width.unwrap_or(32),
                self.decl.height.unwrap_or(32),
            )
        }
    }

    pub fn as_work_href(&self) -> &str {
        self.work_href.as_str()
    }
}

impl AsRef<Asset> for Asset {
    fn as_ref(&self) -> &Asset {
        &self
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(dead_code)]
pub struct AssetDeclaration {
    /// target URL modulo hashing
    href:   Option<String>,
    name:   Option<String>,
    #[serde(default)]
    flags:  Vec<String>,
    #[serde(default)]
    tags:   Vec<String>,
    width:  Option<u32>,
    height: Option<u32>,
    alt:    Option<String>,
}

impl AssetDeclaration {
    pub fn into_asset<S, P1, P2>(
        self,
        file_name: S,
        source_dir: P1,
        work_dir: P2,
    ) -> Result<(String, Asset), std::io::Error>
    where
        S: AsRef<str>,
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let file_name = file_name.as_ref();
        let source_dir = source_dir.as_ref();
        let source_path = source_dir.join(file_name);

        let file_stem = source_path
            .file_stem()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "missing file stem"))?
            .to_str()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "invalid file stem"))?;

        let asset_name = self.name.as_ref().map_or_else(|| file_stem, |s| s.as_str());
        let path_str = self.href.as_ref().map_or_else(|| asset_name, |s| s.as_str());

        let (target_path, work_path) = if self.flags.iter().any(|v| v == "hash") {
            let bytes = std::fs::read(&source_path)?;
            let hash = seahash::hash(bytes.as_ref());
            let target_path = if let Some(ext) = source_path.extension() {
                let ext = ext.to_str().ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::Other, "invalid file extension")
                })?;
                PathBuf::from(format!("{}-{:x}.{}", path_str, hash, ext))
            } else {
                PathBuf::from(format!("{}-{:x}", path_str, hash))
            };
            let work_path = work_dir.as_ref().join(&target_path);

            std::fs::write(&work_path, bytes)?;

            (target_path, work_path)
        } else {
            let target_path = if let Some(ext) = source_path.extension() {
                let ext = ext.to_str().ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::Other, "invalid file extension")
                })?;
                PathBuf::from(format!("{}.{}", path_str, ext))
            } else {
                PathBuf::from(path_str)
            };

            // when not copying, pass source to trunk
            let work_path = source_dir.join(&target_path);

            (target_path, work_path)
        };

        let work_href = work_path
            .into_os_string()
            .into_string()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", err)))?;
        let target_url = target_path
            .into_os_string()
            .into_string()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", err)))?;
        let tags = self.tags.clone();

        Ok((asset_name.to_string(), Asset { source_path, work_href, target_url, tags, decl: self }))
    }
}

fn normalize_path_relative<P1, P2>(path: P1, relative_to: P2) -> Result<PathBuf, std::io::Error>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let path = path.as_ref();
    let abs_path = path.canonicalize().map_err(|err| {
        error!("path \"{:?}\" can't be resolved", path);
        err
    })?;

    let relative_to = relative_to.as_ref();

    Ok(abs_path
        .strip_prefix(relative_to)
        .map_err(|err| {
            error!("path \"{:?}\" doesn't contain path \"{:?}\"", relative_to, abs_path);
            std::io::Error::new(std::io::ErrorKind::Other, err)
        })?
        .into())
}

fn read_folders<P>(manifest_path: P) -> Result<HashMap<String, AssetFolder>, std::io::Error>
where
    P: AsRef<Path>,
{
    let manifest = std::fs::read_to_string(&manifest_path).map_err(|err| {
        error!("manifest \"{}\" is missing", manifest_path.as_ref().to_str().unwrap());
        err
    })?;

    toml::from_str(manifest.as_str()).map_err(|err| {
        error!("manifest \"{}\" is broken", manifest_path.as_ref().to_str().unwrap());
        std::io::Error::new(std::io::ErrorKind::Other, err)
    })
}

fn collect_assets(
    assets: &mut HashMap<String, Asset>,
    visited: &mut HashSet<PathBuf>,
    folders: HashMap<String, AssetFolder>,
    root_dir: &Path,
    work_dir: &Path,
) -> Result<(), std::io::Error> {
    for (folder_path, folder) in folders {
        if folder_path.ends_with(".toml") {
            let manifest_path = root_dir.join(folder_path.as_str());

            if !visited.contains(&manifest_path) {
                let folders = read_folders(&manifest_path)?;
                println!("More folders: {:?}", folders);

                if let Some(base_dir) = manifest_path.parent() {
                    let base_dir = base_dir.canonicalize().map_err(|err| {
                        error!("base dir can't be resolved");
                        err
                    })?;

                    visited.insert(manifest_path);
                    collect_assets(assets, visited, folders, &base_dir, work_dir)?;
                } else {
                    visited.insert(manifest_path);
                    collect_assets(assets, visited, folders, root_dir, work_dir)?;
                }
            }
        } else {
            let folder_path = root_dir.join(folder_path);
            let current_dir = std::env::current_dir().map_err(|err| {
                error!("current dir is unknown");
                err
            })?;
            let source_dir = normalize_path_relative(folder_path, current_dir)?;

            for (asset_name, asset_decl) in folder.0 {
                let (key, asset) = asset_decl.into_asset(asset_name, &source_dir, work_dir)?;

                assets.insert(key, asset);
            }
        }
    }

    Ok(())
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct AssetFolder(HashMap<String, AssetDeclaration>);

#[derive(Serialize, Clone, Default, Debug)]
pub struct AssetGroup {
    name:        String,
    root_dir:    PathBuf,
    current_dir: PathBuf,
    work_dir:    PathBuf,
    assets:      HashMap<String, Asset>, // keyed by file_stem (by default)
}

impl AssetGroup {
    pub fn new<S, P>(group_name: S, manifest_path: P) -> Result<Self, std::io::Error>
    where
        S: AsRef<str>,
        P: AsRef<Path>,
    {
        let name = group_name.as_ref().to_string();

        let folders = read_folders(&manifest_path)?;
        println!("Folders: {:?}", folders);

        let current_dir = std::env::current_dir().map_err(|err| {
            error!("current dir is unknown");
            err
        })?;

        let root_dir = if let Some(root_dir) = manifest_path.as_ref().parent() {
            root_dir.canonicalize().map_err(|err| {
                error!("root dir can't be resolved");
                err
            })?
        } else {
            current_dir.clone()
        };

        let out_dir = std::env::var_os("OUT_DIR").unwrap_or_else(|| {
            warn!("\"OUT_DIR\" isn't set");
            current_dir.as_os_str().into()
        });

        let work_dir = Path::new(&out_dir).strip_prefix(&current_dir).map_err(|err| {
            error!("current dir doesn't contain \"OUT_DIR\"");
            std::io::Error::new(std::io::ErrorKind::Other, err)
        })?;

        let mut assets = HashMap::new();
        let mut visited = HashSet::new();

        collect_assets(&mut assets, &mut visited, folders, &root_dir, &work_dir)?;
        println!("Assets: {:?}", assets);

        Ok(AssetGroup { name, root_dir, current_dir, work_dir: work_dir.into(), assets })
    }

    pub fn extend<S, A, I>(&mut self, assets: I)
    where
        I: IntoIterator<Item=(S, A)>,
        S: AsRef<str>,
        A: AsRef<Asset>,
    {
        for (key, asset) in assets.into_iter() {
            let key = key.as_ref().to_string();
            let asset = asset.as_ref().clone();

            self.assets.insert(key, asset);
        }
    }

    pub fn extend_with_prefix<S1, S2, A, I>(&mut self, prefix: S1, assets: I)
    where
        S1: AsRef<str>,
        I: IntoIterator<Item=(S2, A)>,
        S2: AsRef<str>,
        A: AsRef<Asset>,
    {
        let prefix = prefix.as_ref();

        for (key, asset) in assets.into_iter() {
            let key = format!("{}::{}", prefix, key.as_ref());
            let asset = asset.as_ref().clone();

            self.assets.insert(key, asset);
        }
    }

    pub fn save_mod_files<I>(&self, tags: I) -> Result<(), std::io::Error>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        for tag in tags.into_iter() {
            let tag = tag.as_ref();
            let file_name = format!("{}_{}.rs", self.name, tag);
            let path = Path::new(&self.work_dir).join(file_name);
            let file = std::fs::File::create(path)?;

            writeln!(
                &file,
                "\
use maple_core::{{template, template_result::TemplateResult, generic_node::GenericNode}};"
            )?;

            for (name, asset) in self.assets.iter() {
                if asset.decl.tags.iter().any(|v| v == "img") {
                    writeln!(
                        &file,
                        "
pub fn {}_{}<G: GenericNode>() -> TemplateResult<G> {{
    template! {{ {} }}
}}",
                        name,
                        tag,
                        asset.as_html_element(tag)?,
                    )?;
                }
            }
        }

        Ok(())
    }

    pub fn create_asset<S, P>(
        &self,
        file_name: S,
        source_dir: P,
        decl: AssetDeclaration,
    ) -> Result<(String, Asset), std::io::Error>
    where
        S: AsRef<str>,
        P: AsRef<Path>,
    {
        let source_dir = self.root_dir.join(source_dir);
        let source_dir = normalize_path_relative(source_dir, &self.current_dir)?;

        decl.into_asset(file_name.as_ref(), source_dir, &self.work_dir)
    }

    pub fn register_asset<S: AsRef<str>>(&mut self, key: S, asset: Asset) {
        self.assets.insert(key.as_ref().to_string(), asset);
    }
}

pub trait AssetMaker {
    fn save_html_file<P: AsRef<Path>>(
        &self,
        template: &str,
        out_path: P,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

impl AssetMaker for AssetGroup {
    fn save_html_file<P: AsRef<Path>>(
        &self,
        template: &str,
        out_path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = std::fs::File::create(out_path.as_ref())?;

        let mut tt = TinyTemplate::new();
        tt.add_formatter("links_formatter", links_formatter);
        tt.add_formatter("scripts_formatter", scripts_formatter);
        tt.add_template("html", template)?;

        let rendered = tt.render("html", &self)?;
        writeln!(&file, "{}", rendered)?;

        Ok(())
    }
}

impl AssetMaker for [AssetGroup] {
    fn save_html_file<P: AsRef<Path>>(
        &self,
        template: &str,
        out_path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = std::fs::File::create(out_path.as_ref())?;

        let mut tt = TinyTemplate::new();
        tt.add_formatter("links_formatter", links_formatter);
        tt.add_formatter("scripts_formatter", scripts_formatter);
        tt.add_template("html", template)?;

        let context = if let Some((head, tail)) = self.split_first() {
            let mut context = head.clone();

            for group in tail {
                context.extend_with_prefix(&group.name, &group.assets);
            }

            context
        } else {
            AssetGroup::default()
        };

        let rendered = tt.render("html", &context)?;
        writeln!(&file, "{}", rendered)?;

        Ok(())
    }
}

fn assets_formatter<F1, F2>(
    value: &serde_json::Value,
    output: &mut String,
    filter_fn: F1,
    mut print_fn: F2,
) -> Result<(), TTError>
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
                return Err(TTError::GenericError {
                    msg: format!("Expected a map, found {:?}.", asset),
                })
            };

            let tags = if let Some(tags) = asset.get("tags") {
                if let serde_json::Value::Array(tags) = tags {
                    tags.as_slice()
                } else {
                    return Err(TTError::GenericError {
                        msg: format!("Expected an array, found {:?}.", tags),
                    })
                }
            } else {
                return Err(TTError::GenericError { msg: "Missing tags in asset".to_string() })
            };

            if filter_fn(tags) {
                let work_href = if let Some(work_href) = asset.get("work_href") {
                    if let serde_json::Value::String(work_href) = work_href {
                        work_href.as_str()
                    } else {
                        return Err(TTError::GenericError {
                            msg: "Invalid work_href in asset".to_string(),
                        })
                    }
                } else {
                    return Err(TTError::GenericError {
                        msg: "Missing work_href in asset".to_string(),
                    })
                };

                let target_url = if let Some(target_url) = asset.get("target_url") {
                    if let serde_json::Value::String(target_url) = target_url {
                        target_url.as_str()
                    } else {
                        return Err(TTError::GenericError {
                            msg: "Invalid target_url in asset".to_string(),
                        })
                    }
                } else {
                    return Err(TTError::GenericError {
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
        Err(TTError::GenericError { msg: format!("Expected a map, found {:?}.", value) })
    }
}

fn links_formatter(value: &serde_json::Value, output: &mut String) -> Result<(), TTError> {
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

fn scripts_formatter(value: &serde_json::Value, output: &mut String) -> Result<(), TTError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dotted_file_stem() {
        let decl: AssetDeclaration = toml::from_str("").expect("declaration parsing error");
        let current_dir = std::env::current_dir().expect("current dir is unknown");
        let source_dir = normalize_path_relative(".", current_dir).unwrap();
        let (stem, asset) = decl
            .into_asset("dotted.file.name.ext", source_dir, "work_dir")
            .expect("asset creation error");
        assert_eq!(stem.as_str(), "dotted.file.name");
        assert_eq!(asset.target_url.as_str(), "dotted.file.name.ext");
    }
}
