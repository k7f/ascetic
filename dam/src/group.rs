use std::{
    path::{Path, PathBuf},
    io::Write,
};
use indexmap::IndexMap;
use serde::Serialize;
use tinytemplate::TinyTemplate;
use crate::{
    Asset, AssetDeclaration, AssetMaker, AssetError, sort_assets,
    source::{AssetFolders, AssetPaths},
    formatter::{
        link_assets_formatter, script_assets_formatter, elements_formatter,
        import_assets_formatter, extend_assets_formatter,
    },
};

/// # Examples
///
/// ```no_run
/// use ascetic_dam::{AssetGroup, AssetMaker};
///
/// let asset_group = AssetGroup::new("assets", "assets/Assets.toml").unwrap();
///
/// asset_group.save_all(include_str!("assets/index.tt.html"), "index.trunk.html", &["img"]).unwrap();
///
/// println!("cargo:rerun-if-changed=assets");
/// ```
///
/// or
///
/// ```no_run
/// use ascetic_dam::{AssetGroup, AssetMaker};
///
/// let icon_group = AssetGroup::new("icons", "assets/icons/Assets.toml").unwrap();
/// let badge_group = AssetGroup::new("badges", "assets/badges/Assets.toml").unwrap();
/// let style_group = AssetGroup::new("styles", "assets/styles/Assets.toml").unwrap();
/// let script_group = AssetGroup::new("scripts", "assets/scripts/Assets.toml").unwrap();
///
/// icon_group.save_mod_files(&["img"]).unwrap();
/// badge_group.save_mod_files(&["img"]).unwrap();
///
/// [icon_group, badge_group, style_group, script_group]
///     .save_html_file(include_str!("assets/index.tt.html"), "index.trunk.html")
///     .unwrap();
///
/// println!("cargo:rerun-if-changed=assets");
/// ```
///
/// or
///
/// ```no_run
/// use ascetic_dam::{AssetGroup, AssetMaker};
///
/// let icon_group = AssetGroup::new("icons", "assets/icons/Assets.toml").unwrap();
/// let badge_group = AssetGroup::new("badges", "assets/badges/Assets.toml").unwrap();
/// let style_group = AssetGroup::new("styles", "assets/styles/Assets.toml").unwrap();
/// let script_group = AssetGroup::new("scripts", "assets/scripts/Assets.toml").unwrap();
///
/// [icon_group, badge_group, style_group, script_group]
///     .save_all(include_str!("assets/index.tt.html"), "index.trunk.html", &["img"])
///     .unwrap();
///
/// println!("cargo:rerun-if-changed=assets");
/// ```
#[derive(Serialize, Clone, Default, Debug)]
pub struct AssetGroup {
    name:     String,
    title:    Option<String>,
    paths:    AssetPaths,
    elements: Vec<Asset>,
    assets:   IndexMap<String, Asset>, // keyed by file_stem (by default)
}

impl AssetGroup {
    pub fn new<S, P>(group_name: S, manifest_path: P) -> Result<Self, AssetError>
    where
        S: AsRef<str>,
        P: AsRef<Path>,
    {
        let name = group_name.as_ref().to_string();
        let title = None;
        let paths = AssetPaths::from_manifest(&manifest_path)?;
        let folders = AssetFolders::from_manifest(&manifest_path)?;
        println!("\n*** Folders ***\n{:#?}", folders);

        let elements = Vec::new();
        let mut assets = folders.collect_assets(&paths)?;

        sort_assets(&mut assets);
        println!("\n*** Assets ***\n{:#?}", assets);

        Ok(AssetGroup { name, title, paths, elements, assets })
    }

    pub fn as_empty(&self) -> Self {
        let name = self.name.clone();
        let title = self.title.clone();
        let paths = self.paths.clone();
        let elements = Vec::new();
        let assets = IndexMap::new();

        AssetGroup { name, title, paths, elements, assets }
    }

    pub fn clone_with_prefix<S>(&self, prefix: S) -> Self
    where
        S: AsRef<str>,
    {
        let mut result = self.as_empty();

        result.extend_with_prefix(prefix, &self.assets);

        result
    }

    pub fn with_title<S: AsRef<str>>(mut self, title: S) -> Self {
        self.set_title(title);
        self
    }

    #[inline]
    pub fn set_title<S: AsRef<str>>(&mut self, title: S) {
        self.title = Some(title.as_ref().to_string());
    }

    pub fn with_elements<I>(mut self, elts: I) -> Self
    where
        I: IntoIterator<Item = Asset>,
    {
        self.elements.extend(elts);
        self
    }

    #[inline]
    pub fn num_assets(&self) -> usize {
        self.assets.len()
    }

    #[inline]
    pub fn get_assets(&self) -> indexmap::map::Iter<String, Asset> {
        self.assets.iter()
    }

    #[inline]
    pub fn work_dir(&self) -> &PathBuf {
        self.paths.work_dir()
    }

    #[inline]
    pub fn rooted_path<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf, AssetError> {
        self.paths.rooted_path(path)
    }

    #[inline]
    pub fn has_flag<S>(&self, flag: S) -> bool
    where
        S: AsRef<str>,
    {
        let flag = flag.as_ref();
        self.assets.values().any(|asset| asset.get_flags().any(|v| v == flag))
    }

    #[inline]
    pub fn has_tag<S>(&self, tag: S) -> bool
    where
        S: AsRef<str>,
    {
        let tag = tag.as_ref();
        self.assets.values().any(|asset| asset.get_tags().any(|v| v == tag))
    }

    pub fn extend<S, A, I>(&mut self, assets: I)
    where
        I: IntoIterator<Item = (S, A)>,
        S: AsRef<str>,
        A: AsRef<Asset>,
    {
        for (key, asset) in assets.into_iter() {
            let key = key.as_ref().to_string();
            let asset = asset.as_ref().clone();

            self.assets.insert(key, asset);
        }

        self.sort();
    }

    pub fn extend_with_prefix<S1, S2, A, I>(&mut self, prefix: S1, assets: I)
    where
        S1: AsRef<str>,
        I: IntoIterator<Item = (S2, A)>,
        S2: AsRef<str>,
        A: AsRef<Asset>,
    {
        let prefix = prefix.as_ref();

        for (key, asset) in assets.into_iter() {
            let key = format!("{}::{}", prefix, key.as_ref());
            let asset = asset.as_ref().clone();

            self.assets.insert(key, asset);
        }

        self.sort();
    }

    pub fn render_html_template(&self, template: &str) -> Result<String, AssetError> {
        let mut tt = TinyTemplate::new();

        tt.add_formatter("link_assets_formatter", link_assets_formatter);
        tt.add_formatter("script_assets_formatter", script_assets_formatter);
        tt.add_formatter("elements_formatter", elements_formatter);
        tt.add_template("html", template)?;
        let result = tt.render("html", self)?;

        Ok(result)
    }

    pub fn render_scss_template(&self, template: &str) -> Result<String, AssetError> {
        let mut tt = TinyTemplate::new();

        tt.add_formatter("import_assets_formatter", import_assets_formatter);
        tt.add_formatter("extend_assets_formatter", extend_assets_formatter);
        tt.add_template("scss", template)?;
        let result = tt.render("scss", self)?;

        Ok(result)
    }

    pub fn create_asset<S, P>(
        &self,
        file_name: S,
        source_dir: P,
        decl: AssetDeclaration,
    ) -> Result<(String, Asset), AssetError>
    where
        S: AsRef<str>,
        P: AsRef<Path>,
    {
        let source_dir = self.paths.rooted_path(source_dir)?;
        let work_dir = self.paths.work_dir();

        decl.into_asset(file_name.as_ref(), source_dir, work_dir)
    }

    pub fn register_asset<S: AsRef<str>>(
        &mut self,
        key: S,
        asset: Asset,
    ) -> Result<(), AssetError> {
        let key = key.as_ref();

        if self.assets.insert(key.to_string(), asset).is_some() {
            Err(AssetError::asset_key_clash(key))
        } else {
            self.sort();
            Ok(())
        }
    }

    #[inline]
    fn sort(&mut self) {
        sort_assets(&mut self.assets);
    }
}

impl AssetMaker for AssetGroup {
    fn as_group(&self) -> AssetGroup {
        self.clone()
    }

    fn save_mod_files<I>(&self, tags: I) -> Result<(), AssetError>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let file_name = format!("{}_url.rs", self.name);
        let path = Path::new(self.paths.work_dir()).join(file_name);
        let file = std::fs::File::create(path)?;

        for (name, asset) in self.assets.iter() {
            writeln!(
                &file,
                "
pub fn {}_url() -> &'static str {{
    \"{}\"
}}",
                name,
                asset.as_target_url(),
            )?;
        }

        for tag in tags.into_iter().filter(|tag| self.has_tag(tag)) {
            let tag = tag.as_ref();
            let file_name = format!("{}_{}.rs", self.name, tag);
            let path = Path::new(self.paths.work_dir()).join(file_name);
            let file = std::fs::File::create(path)?;

            writeln!(
                &file,
                "\
use maple_core::{{template, template_result::TemplateResult, generic_node::GenericNode}};"
            )?;

            for (name, asset) in self.assets.iter() {
                if asset.get_tags().any(|v| v == tag) {
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

    fn save_html_file<P: AsRef<Path>>(
        &self,
        template: &str,
        out_path: P,
    ) -> Result<(), AssetError> {
        let file = std::fs::File::create(out_path.as_ref())?;
        let rendered = self.render_html_template(template)?;

        writeln!(&file, "{}", rendered)?;

        Ok(())
    }
}

impl AssetMaker for [AssetGroup] {
    fn as_group(&self) -> AssetGroup {
        if let Some((head, tail)) = self.split_first() {
            let mut result = head.clone_with_prefix(&head.name);

            for group in tail {
                result.extend_with_prefix(&group.name, &group.assets);
            }

            result
        } else {
            AssetGroup::default()
        }
    }

    fn save_mod_files<I>(&self, tags: I) -> Result<(), AssetError>
    where
        I: IntoIterator + Clone,
        I::Item: AsRef<str>,
    {
        for group in self.iter() {
            group.save_mod_files(tags.clone())?;
        }

        Ok(())
    }
}

impl AssetMaker for Vec<Result<AssetGroup, AssetError>> {
    fn as_group(&self) -> AssetGroup {
        let mut iter = self.iter().filter_map(|item| item.as_ref().ok());

        if let Some(head) = iter.next() {
            let mut result = head.clone_with_prefix(&head.name);

            for group in iter {
                result.extend_with_prefix(&group.name, &group.assets);
            }

            result
        } else {
            AssetGroup::default()
        }
    }

    fn save_mod_files<I>(&self, tags: I) -> Result<(), AssetError>
    where
        I: IntoIterator + Clone,
        I::Item: AsRef<str>,
    {
        for group in self.iter().filter_map(|item| item.as_ref().ok()) {
            group.save_mod_files(tags.clone())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::declaration_from_spec;

    #[test]
    fn group_title() {
        let group = AssetGroup::default().with_title("Test");
        let rendered = group.render_html_template(r#"{title}"#);
        assert_eq!(rendered.unwrap().as_str(), "Test");
    }

    #[test]
    fn key_clash() {
        let mut g1 = AssetGroup::default();
        let (_, asset) = g1
            .create_asset("file_name.ext", ".", declaration_from_spec(""))
            .expect("asset creation error");
        g1.register_asset("test", asset).expect("asset registration error");
        g1.name = "g1".to_string();

        let mut g2 = AssetGroup::default();
        let (_, asset) = g2
            .create_asset("file_name.ext", ".", declaration_from_spec(""))
            .expect("asset creation error");
        g2.register_asset("test", asset).expect("asset registration error");
        g2.name = "g2".to_string();

        let group = [g1, g2].as_group();
        assert!(group.assets.contains_key("g1::test"));
        assert!(group.assets.contains_key("g2::test"));
    }
}
