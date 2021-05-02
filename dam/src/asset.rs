use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    io::Write,
    sync::atomic::{self, AtomicU64},
};
use tracing::error;
use indexmap::IndexMap;
use serde::{Serialize, Deserialize};
use crate::{AssetGroup, AssetError, detailed_error, error::DetailedError};

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
    flags:         Vec<String>,
    tags:          Vec<String>,
    attrs:         HashMap<String, String>, // maps tags to attribute lists
    extends:       Vec<String>,
    #[serde(skip_serializing)]
    decl:          AssetDeclaration,
}

impl Asset {
    pub fn get_flags(&self) -> std::slice::Iter<String> {
        self.flags.iter()
    }

    pub fn get_tags(&self) -> std::slice::Iter<String> {
        self.tags.iter()
    }

    pub fn get_extends(&self) -> std::slice::Iter<String> {
        self.extends.iter()
    }

    pub fn as_html_element<S: AsRef<str>>(&self, tag: S) -> Result<String, AssetError> {
        let tag = tag.as_ref();
        match tag {
            "img" => Ok(self.as_img()),
            _ => Err(AssetError::tag_unrenderable(tag)),
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

    pub fn as_target_url(&self) -> &str {
        self.target_url.as_str()
    }
}

impl AsRef<Asset> for Asset {
    fn as_ref(&self) -> &Asset {
        &self
    }
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct AssetDeclaration {
    /// target URL modulo hashing
    href:    Option<String>,
    name:    Option<String>,
    #[serde(default)]
    flags:   Vec<String>,
    #[serde(default)]
    tags:    Vec<String>,
    #[serde(default)]
    attrs:   HashMap<String, String>,
    width:   Option<u32>,
    height:  Option<u32>,
    alt:     Option<String>,
    #[serde(default)]
    extends: Vec<String>,
}

impl AssetDeclaration {
    pub fn with_flags<I>(mut self, flags: I) -> Result<Self, AssetError>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        for flag in flags.into_iter() {
            self.add_flag(flag)?
        }

        Ok(self)
    }

    pub fn with_tags<I>(mut self, tags: I) -> Result<Self, AssetError>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        for tag in tags.into_iter() {
            self.add_tag(tag)?
        }

        Ok(self)
    }

    pub fn with_extends<I>(mut self, extends: I) -> Result<Self, AssetError>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        for class_name in extends.into_iter() {
            self.add_class_name(class_name)?
        }

        Ok(self)
    }

    pub fn add_flag<S>(&mut self, flag: S) -> Result<(), AssetError>
    where
        S: AsRef<str>,
    {
        let flag = flag.as_ref();

        if self.flags.iter().any(|f| flag == f.as_str()) {
            Err(AssetError::flag_clash(flag))
        } else {
            self.flags.push(flag.to_string());
            Ok(())
        }
    }

    pub fn add_tag<S>(&mut self, tag: S) -> Result<(), AssetError>
    where
        S: AsRef<str>,
    {
        let tag = tag.as_ref();

        if self.tags.iter().any(|t| tag == t.as_str()) {
            Err(AssetError::tag_clash(tag))
        } else {
            self.tags.push(tag.to_string());
            Ok(())
        }
    }

    pub fn add_class_name<S>(&mut self, class_name: S) -> Result<(), AssetError>
    where
        S: AsRef<str>,
    {
        let class_name = class_name.as_ref();

        if self.extends.iter().all(|n| class_name != n.as_str()) {
            self.extends.push(class_name.to_string());
        }

        Ok(())
    }

    pub fn add_attrs<S1, S2>(&mut self, tag: S1, attrs: S2)
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        let entry = self.attrs.entry(tag.as_ref().to_string()).or_insert_with(String::new);
        entry.push_str(attrs.as_ref());
    }

    pub fn into_asset<S, P1, P2>(
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
        let flags = self.flags.clone();
        let tags = self.tags.clone();
        let attrs = self.attrs.clone();
        let extends = self.extends.clone();

        Ok((
            asset_name.to_string(),
            Asset {
                serial_number,
                source_path,
                work_href,
                target_url,
                flags,
                tags,
                attrs,
                extends,
                decl: self,
            },
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
        let rendered = context.render_html_template(template)?;

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

#[cfg(test)]
mod tests {
    use crate::tests::{create_dummy_asset, asset_from_spec};

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
