use std::env;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::aspects;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub mounts: Option<Vec<aspects::Mount>>,
}

impl Config {
    pub fn empty() -> Config {
        Config { mounts: None }
    }

    pub fn save(
        &self,
        application: Option<&str>,
        profile: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let config_dir = get_config_dir(application, profile);
        fs::create_dir_all(&config_dir)?;

        let path = config_dir.join("config.yaml");
        let mut config_file = fs::File::create(path)?;

        let s = serde_yaml::to_string(&self)?;
        config_file.write_all(&s.into_bytes())?;

        Ok(())
    }

    /// Loads a single config file specified by the combination of application and profile options;
    /// if both are none, then loads the global config.
    fn load_layer(
        application: Option<&str>,
        profile: Option<&str>,
    ) -> Result<Config, Box<dyn Error>> {
        let config_dir = get_config_dir(application, profile);
        let yaml_file = config_dir.join("config.yaml");

        let mut cfg = Config::empty();

        if yaml_file.exists() {
            let yaml = fs::read_to_string(yaml_file)?;
            cfg = serde_yaml::from_str(&yaml)?;
        }

        Ok(cfg)
    }

    pub fn load(application: &str, profile: Option<&str>) -> Result<Config, Box<dyn Error>> {
        // load dfiles global config if it exists
        let global_config = Config::load_layer(None, None)?;
        // load application global config if it exists
        let app_config = Config::load_layer(Some(application), None)?;
        // load application profile config if profile is specified and it exists
        let profile_config = Config::load_layer(Some(application), profile)?;
        Ok(global_config.merge(&app_config).merge(&profile_config))
    }

    /// destructively merge values from other onto a copy of self, producing a new Config
    fn merge(&self, other: &Config) -> Config {
        let mut cfg = (*self).clone();

        if let Some(v) = &other.mounts {
            cfg.mounts = Some(v.clone());
        }

        cfg
    }

    pub fn get_aspects(&self) -> Vec<Box<dyn aspects::ContainerAspect>> {
        let mut aspects: Vec<Box<dyn aspects::ContainerAspect>> = Vec::new();

        if let Some(mounts) = &self.mounts {
            for mount in mounts {
                aspects.push(Box::new(mount.clone()));
            }
        }

        aspects
    }
}

fn get_config_dir(application: Option<&str>, profile: Option<&str>) -> PathBuf {
    let home = env::var("HOME").expect("HOME var must be set");
    let mut config_dir = Path::new(&home).join(".config").join("dfiles");

    if let Some(s) = application {
        config_dir = config_dir.join("applications").join(s)
    }

    if let Some(s) = profile {
        config_dir = config_dir.join("profiles").join(s)
    }
    return config_dir;
}
