use std::{
    path::{Path, PathBuf},
    collections::HashSet,
};
use tracing::{warn, error};
use indexmap::{self, IndexMap};
use serde::{Serialize, Deserialize};
use crate::{
    Asset, AssetDeclaration, detailed_error,
    error::{AssetError, DetailedError},
};

#[derive(Serialize, Clone, Default, Debug)]
pub(crate) struct AssetPaths {
    root_dir:    PathBuf,
    current_dir: PathBuf,
    work_dir:    PathBuf,
}

impl AssetPaths {
    pub(crate) fn from_manifest<P: AsRef<Path>>(manifest_path: P) -> Result<Self, AssetError> {
        let current_dir =
            std::env::current_dir().map_err(detailed_error!("current dir is unknown"))?;
        let root_dir = if let Some(root_dir) = manifest_path.as_ref().parent() {
            root_dir
                .canonicalize()
                .map_err(detailed_error!("root dir {:?} can't be resolved", root_dir))?
        } else {
            current_dir.clone()
        };

        let out_dir = std::env::var_os("OUT_DIR").unwrap_or_else(|| {
            warn!("\"OUT_DIR\" isn't set");
            current_dir.as_os_str().into()
        });

        let work_dir = Path::new(&out_dir)
            .strip_prefix(&current_dir)
            .map_err(AssetError::std_io)
            .map_err(detailed_error!("current dir {:?} doesn't contain \"OUT_DIR\"", current_dir))?
            .into();

        Ok(AssetPaths { root_dir, current_dir, work_dir })
    }

    #[inline]
    pub(crate) fn work_dir(&self) -> &PathBuf {
        &self.work_dir
    }

    pub(crate) fn normalize_path<P1, P2>(path: P1, relative_to: P2) -> Result<PathBuf, AssetError>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let path = path.as_ref();
        let abs_path =
            path.canonicalize().map_err(detailed_error!("path {:?} can't be resolved", path))?;
        let relative_to = relative_to.as_ref();

        Ok(abs_path
            .strip_prefix(relative_to)
            .map_err(AssetError::std_io)
            .map_err(detailed_error!("path {:?} doesn't contain path {:?}", relative_to, abs_path))?
            .into())
    }

    #[inline]
    pub(crate) fn rooted_path<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf, AssetError> {
        Self::normalize_path(self.root_dir.join(path), &self.current_dir)
    }
}

type AssetFolder = IndexMap<String, AssetDeclaration>;

#[derive(Deserialize, Debug)]
pub(crate) struct AssetFolders(IndexMap<String, AssetFolder>);

impl AssetFolders {
    pub(crate) fn from_manifest<P: AsRef<Path>>(manifest_path: P) -> Result<Self, AssetError> {
        let manifest = std::fs::read_to_string(&manifest_path).map_err(detailed_error!(
            "manifest \"{}\" is missing",
            manifest_path.as_ref().to_str().unwrap()
        ))?;

        let folders = toml::from_str(manifest.as_str()).map_err(AssetError::std_io).map_err(
            detailed_error!("manifest \"{}\" is broken", manifest_path.as_ref().to_str().unwrap()),
        )?;

        Ok(AssetFolders(folders))
    }

    // Returns unordered assets!
    pub(crate) fn collect_assets(
        &self,
        paths: &AssetPaths,
    ) -> Result<IndexMap<String, Asset>, AssetError> {
        let mut assets = IndexMap::new();
        let mut visited = HashSet::new();

        let count =
            self.collect_impl(&mut assets, &mut visited, &paths.root_dir, &paths.work_dir)?;

        if count == assets.len() {
            Ok(assets)
        } else {
            Err(AssetError::mismatched_collect(count, assets.len()))
        }
    }

    fn collect_impl(
        &self,
        assets: &mut IndexMap<String, Asset>,
        visited: &mut HashSet<PathBuf>,
        root_dir: &Path,
        work_dir: &Path,
    ) -> Result<usize, AssetError> {
        let mut running_count = 0;

        for (folder_path, folder) in self.0.iter() {
            if folder_path.ends_with(".toml") {
                let manifest_path = root_dir.join(folder_path.as_str());

                if !visited.contains(&manifest_path) {
                    let folders = AssetFolders::from_manifest(&manifest_path)?;
                    println!("\n*** More folders ***\n{:#?}", folders);

                    if let Some(base_dir) = manifest_path.parent() {
                        let base_dir = base_dir.canonicalize().map_err(detailed_error!(
                            "base dir {:?} can't be resolved",
                            base_dir
                        ))?;

                        visited.insert(manifest_path);
                        running_count +=
                            folders.collect_impl(assets, visited, &base_dir, work_dir)?;
                    } else {
                        visited.insert(manifest_path);
                        running_count +=
                            folders.collect_impl(assets, visited, root_dir, work_dir)?;
                    }
                }
            } else {
                let folder_path = root_dir.join(folder_path);
                let current_dir =
                    std::env::current_dir().map_err(detailed_error!("current dir is unknown"))?;
                let source_dir = AssetPaths::normalize_path(folder_path, current_dir)?;

                for (asset_name, asset_decl) in folder.iter() {
                    let (key, asset) =
                        asset_decl.clone().into_asset(asset_name, &source_dir, work_dir)?;

                    assets.insert(key, asset);
                    running_count += 1;
                }
            }
        }

        Ok(running_count)
    }
}
