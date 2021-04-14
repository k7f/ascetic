// FIXME(#0) push scripts to end of <body>

use std::{
    path::{Path, PathBuf},
    io::Write,
    sync::atomic::{self, AtomicU64},
};
use tracing::error;
use indexmap::{self, IndexMap};
use serde::{Serialize, Deserialize};

pub use ascetic_dam_macro::assets;

mod source;
mod group;
mod formatter;
mod error;

pub use group::AssetGroup;
pub use error::AssetError;

use error::DetailedError;

static ASSET_SERIAL_NUMBER: AtomicU64 = AtomicU64::new(0);

#[inline]
pub fn sort_assets(assets: &mut IndexMap<String, Asset>) {
    assets.sort_by(|_, v1, _, v2| v1.serial_number.cmp(&v2.serial_number));
}

/// Every path in the call graph leading to `Asset` creation is
/// expected to go through `AssetDeclaration::into_asset()`.
#[derive(Serialize, Clone, Debug)]
pub struct Asset {
    serial_number: u64,
    source_path:   PathBuf,
    work_href:     String,
    target_url:    String,
    tags:          Vec<String>,
    #[serde(skip_serializing)]
    decl:          AssetDeclaration,
}

impl Asset {
    pub fn as_html_element<S: AsRef<str>>(&self, tag: S) -> Result<String, AssetError> {
        let tag = tag.as_ref();
        match tag {
            "img" => Ok(self.as_img()),
            _ => Err(AssetError::bad_tag(tag)),
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
    fn into_asset<S, P1, P2>(
        self,
        file_name: S,
        source_dir: P1,
        work_dir: P2,
    ) -> Result<(String, Asset), AssetError>
    where
        S: AsRef<str>,
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let serial_number = ASSET_SERIAL_NUMBER.fetch_add(1, atomic::Ordering::SeqCst);

        let file_name = file_name.as_ref();
        let source_dir = source_dir.as_ref();
        let source_path = source_dir.join(file_name);

        let file_stem = source_path
            .file_stem()
            .ok_or_else(|| AssetError::std_io("Missing file stem"))?
            .to_str()
            .ok_or_else(|| AssetError::std_io("Invalid file stem"))?;

        let asset_name = self.name.as_ref().map_or_else(|| file_stem, |s| s.as_str());
        let path_str = self.href.as_ref().map_or_else(|| asset_name, |s| s.as_str());

        let (target_path, work_path) = if self.flags.iter().any(|v| v == "hash") {
            let bytes = std::fs::read(&source_path)
                .map_err(|err| err.with_string(format!("missing file {:?}", source_path)))?;
            let hash = seahash::hash(bytes.as_ref());
            let target_path = if let Some(ext) = source_path.extension() {
                let ext =
                    ext.to_str().ok_or_else(|| AssetError::std_io("Invalid file extension"))?;
                PathBuf::from(format!("{}-{:x}.{}", path_str, hash, ext))
            } else {
                PathBuf::from(format!("{}-{:x}", path_str, hash))
            };
            let work_path = work_dir.as_ref().join(&target_path);

            std::fs::write(&work_path, bytes)?;

            (target_path, work_path)
        } else {
            let target_path = if let Some(ext) = source_path.extension() {
                let ext = ext
                    .to_str()
                    .ok_or_else(|| AssetError::std_io("Invalid file extension"))
                    .map_err(detailed_error!("{:?}", ext))?;
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
            .map_err(|err| AssetError::std_io(format!("{:?}", err)))
            .map_err(detailed_error!("Path error"))?;
        let target_url = target_path
            .into_os_string()
            .into_string()
            .map_err(|err| AssetError::std_io(format!("{:?}", err)))
            .map_err(detailed_error!("Path error"))?;
        let tags = self.tags.clone();

        Ok((
            asset_name.to_string(),
            Asset { serial_number, source_path, work_href, target_url, tags, decl: self },
        ))
    }
}

pub trait AssetMaker {
    fn as_group(&self) -> AssetGroup;

    fn save_mod_files<I>(&self, tags: I) -> Result<(), AssetError>
    where
        I: IntoIterator + Clone,
        I::Item: AsRef<str>;

    fn save_html_file<P: AsRef<Path>>(
        &self,
        template: &str,
        out_path: P,
    ) -> Result<(), AssetError> {
        let file = std::fs::File::create(out_path.as_ref())?;
        let context = self.as_group();
        let rendered = context.render_template(template)?;

        writeln!(&file, "{}", rendered)?;

        Ok(())
    }

    fn save_all<P, I>(&self, template: &str, out_path: P, tags: I) -> Result<(), AssetError>
    where
        P: AsRef<Path>,
        I: IntoIterator + Clone,
        I::Item: AsRef<str>,
    {
        self.save_mod_files(tags)?;
        self.save_html_file(template, out_path)
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Default)]
pub struct DAM {
    title:    Option<String>,
    template: Option<String>,
    groups:   Vec<Result<AssetGroup, AssetError>>,
    tags:     Vec<String>,
}

impl DAM {
    pub fn new() -> Self {
        DAM::default()
    }

    pub fn with_title<S>(mut self, title: S) -> Self
    where
        S: AsRef<str>,
    {
        let title = title.as_ref();

        for group in self.groups.iter_mut().filter_map(|g| g.as_mut().ok()) {
            group.set_title(title);
        }
        self.title = Some(title.to_string());

        self
    }

    pub fn with_template<S>(mut self, template: S) -> Self
    where
        S: AsRef<str>,
    {
        self.template = Some(template.as_ref().to_string());
        self
    }

    pub fn with_group<S, P>(mut self, group_name: S, manifest_path: P) -> Self
    where
        S: AsRef<str>,
        P: AsRef<Path>,
    {
        match AssetGroup::new(group_name, manifest_path) {
            Ok(mut group) => {
                if let Some(ref title) = self.title {
                    group.set_title(title);
                }
                self.groups.push(Ok(group));
            }
            err => self.groups.push(err),
        }

        self
    }

    pub fn with_tags<A, S>(mut self, tags: A) -> Self
    where
        A: Into<Vec<S>>,
        S: AsRef<str>,
    {
        self.tags.extend(tags.into().into_iter().map(|tag| tag.as_ref().to_string()));
        self
    }

    pub fn save<'a, P>(&'a self, out_path: P) -> Result<(), Box<dyn std::error::Error + 'a>>
    where
        P: AsRef<Path>,
    {
        self.groups.save_mod_files(&self.tags)?;

        let file = std::fs::File::create(out_path.as_ref())?;
        let context = self.groups.as_group();
        let rendered = if let Some(ref template) = self.template {
            context.render_template(template)
        } else {
            context.render_template(include_str!("assets/index.tt.html"))
        }?;

        writeln!(&file, "{}", rendered)?;
        self.groups.iter().try_for_each(|g| g.as_ref().map(|_| ()).map_err(|err| err.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn create_dummy_asset() -> Asset {
        let decl = declaration_from_spec("");
        let (_, asset) = decl.into_asset("asset.test", "", "").expect("asset creation error");
        asset
    }

    pub(crate) fn declaration_from_spec(spec: &str) -> AssetDeclaration {
        toml::from_str(spec).expect("declaration parsing error")
    }

    pub(crate) fn asset_from_spec(file_name: &str, work_dir: &str, spec: &str) -> (String, Asset) {
        let decl = declaration_from_spec(spec);
        let current_dir = std::env::current_dir().expect("current dir is unknown");
        let source_dir = source::AssetPaths::normalize_path(".", current_dir).unwrap();

        decl.into_asset(file_name, source_dir, work_dir).expect("asset creation error")
    }

    #[test]
    fn test_serial_number() {
        let mut ser_no = create_dummy_asset().serial_number;
        for _ in 0..43 {
            let new_ser_no = create_dummy_asset().serial_number;
            assert!(new_ser_no > ser_no);
            ser_no = new_ser_no;
        }
    }

    #[test]
    fn test_dotted_file_stem() {
        let (stem, asset) = asset_from_spec("dotted.file.name.ext", "work_dir", "");
        assert_eq!(stem.as_str(), "dotted.file.name");
        assert_eq!(asset.target_url.as_str(), "dotted.file.name.ext");
    }
}
