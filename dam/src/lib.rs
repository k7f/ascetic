mod asset;
mod group;
mod source;
mod formatter;
mod error;

pub use ascetic_dam_macro::assets;
pub use asset::{Asset, AssetDeclaration, AssetMaker, sort_assets};
pub use group::AssetGroup;
pub use error::AssetError;

use std::{path::Path, io::Write};

#[allow(clippy::upper_case_acronyms)]
#[derive(Default)]
pub struct DAM {
    title:          Option<String>,
    html_template:  Option<String>,
    scss_template:  Option<String>,
    scss_file_name: Option<String>,
    elements:       AssetGroup,
    groups:         Vec<Result<AssetGroup, AssetError>>,
    tags:           Vec<String>,
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

    pub fn with_element<S1, S2>(mut self, tag: S1, attrs: S2) -> Result<Self, AssetError>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        let tag = tag.as_ref();
        let attrs = attrs.as_ref().trim();

        let mut decl = AssetDeclaration::default().with_tags(Some(tag))?;

        if !attrs.is_empty() {
            decl.add_attrs(tag, attrs);
        }

        let name = format!("{}-element-{}", tag, self.elements.num_assets());
        let (key, asset) = self.elements.create_asset(name, ".", decl)?;
        self.elements.register_asset(key, asset);

        Ok(self)
    }

    pub fn with_html_template<S>(mut self, template: S) -> Self
    where
        S: AsRef<str>,
    {
        self.html_template = Some(template.as_ref().to_string());
        self
    }

    pub fn with_scss_template<S1, S2>(mut self, template: S1, rendered_name: S2) -> Self
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        self.scss_template = Some(template.as_ref().to_string());
        self.scss_file_name = Some(rendered_name.as_ref().to_string());
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

        let mut context =
            self.groups.as_group().with_elements(self.elements.get_assets().map(|v| v.1.clone()));

        if let Some(ref scss_template) = self.scss_template {
            if let Some(ref scss_file_name) = self.scss_file_name {
                let work_dir = context.work_dir();
                let out_path = work_dir.clone().join(scss_file_name);
                let file = std::fs::File::create(out_path)?;
                let rendered = context.render_scss_template(scss_template)?;
                writeln!(&file, "{}", rendered)?;

                let decl: AssetDeclaration = toml::from_str("tags = [\"link\"]")?;
                let (key, asset) = decl.into_asset("index.scss", work_dir, "")?;
                let key = format!("scss_from_tt::{}", key);
                context.register_asset(key, asset);
                println!("{:#?}", context);
            } else {
                return Err(AssetError::missing_target_for_template("SCSS").into())
            }
        }

        let file = std::fs::File::create(out_path.as_ref())?;
        let rendered = if let Some(ref html_template) = self.html_template {
            context.render_html_template(html_template)
        } else {
            context.render_html_template(include_str!("assets/index.tt.html"))
        }?;

        writeln!(&file, "{}", rendered)?;
        self.groups.iter().try_for_each(|g| g.as_ref().map(|_| ()).map_err(|err| err.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn declaration_from_spec(spec: &str) -> AssetDeclaration {
        toml::from_str(spec).expect("declaration parsing error")
    }

    pub(crate) fn create_dummy_asset() -> Asset {
        let decl = declaration_from_spec("");
        let (_, asset) = decl.into_asset("asset.test", "", "").expect("asset creation error");
        asset
    }

    pub(crate) fn asset_from_spec(file_name: &str, work_dir: &str, spec: &str) -> (String, Asset) {
        let decl = declaration_from_spec(spec);
        let current_dir = std::env::current_dir().expect("current dir is unknown");
        let source_dir = source::AssetPaths::normalize_path(".", current_dir).unwrap();

        decl.into_asset(file_name, source_dir, work_dir).expect("asset creation error")
    }
}
