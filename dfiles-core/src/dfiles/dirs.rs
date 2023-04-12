use std::path::PathBuf;

use directories_next::ProjectDirs;

use super::error::{Error, Result};

enum DirType {
    Config,
    Data,
}

pub fn get_config_dir(application: Option<&str>, profile: Option<&str>) -> Result<PathBuf> {
    get_dir(DirType::Config, application, profile)
}

pub fn get_data_dir(application: Option<&str>, profile: Option<&str>) -> Result<PathBuf> {
    get_dir(DirType::Data, application, profile)
}

fn get_dir(dir_type: DirType, application: Option<&str>, profile: Option<&str>) -> Result<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "dfiles") {
        let mut dir = match dir_type {
            DirType::Config => proj_dirs.config_dir().to_path_buf(),
            DirType::Data => proj_dirs.data_dir().to_path_buf(),
        };

        if let Some(s) = application {
            dir = dir.join("applications").join(s);
        }

        if let Some(s) = profile {
            dir = dir.join("profiles").join(s);
        }

        Ok(dir)
    } else {
        Err(Error::MissingDirectory)
    }
}
