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
    pub timezone: Option<aspects::Timezone>,
}

impl Config {
    pub fn empty() -> Config {
        Config {
            mounts: None,
            timezone: None,
        }
    }

    pub fn save(
        &self,
        application: Option<&str>,
        profile: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let existing_config = Config::load_layer(application, profile)?;
        let merged = existing_config.merge(self, true);

        let config_dir = get_config_dir(application, profile);
        fs::create_dir_all(&config_dir)?;

        let path = config_dir.join("config.yaml");
        let mut config_file = fs::File::create(path)?;

        let s = serde_yaml::to_string(&merged)?;
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
        Ok(global_config
            .merge(&app_config, false)
            .merge(&profile_config, false))
    }

    /// destructively merge values from other onto a copy of self, producing a new Config
    fn merge(&self, other: &Config, overwrite: bool) -> Config {
        let mut cfg = (*self).clone();

        cfg.mounts = merge(&self.mounts, &other.mounts, overwrite);

        if let Some(v) = &other.timezone {
            cfg.timezone = Some(v.clone());
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

        if let Some(timezone) = &self.timezone {
            aspects.push(Box::new(timezone.clone()));
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

fn merge<T: Clone>(
    left: &Option<Vec<T>>,
    right: &Option<Vec<T>>,
    overwrite: bool,
) -> Option<Vec<T>> {
    let mut new = Vec::new();

    if let Some(v) = &left {
        new = v.clone();
    }

    if let Some(v) = &right {
        if overwrite {
            new = v.clone();
        } else {
            new.append(&mut v.clone());
        }
    }

    match new.len() {
        x if x > 0 => Some(new),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_returns_some() {
        let left: Vec<i32> = vec![1, 2, 3, 4];
        let right: Vec<i32> = vec![5, 6, 7];
        let empty: Option<Vec<i32>> = Some(Vec::new());

        assert_eq!(
            merge(&Some(left.clone()), &Some(right.clone()), true),
            Some(vec![5, 6, 7])
        );
        assert_eq!(
            merge(&Some(left.clone()), &Some(right.clone()), false),
            Some(vec![1, 2, 3, 4, 5, 6, 7])
        );

        assert_eq!(
            merge(&Some(left.clone()), &None, false),
            Some(vec![1, 2, 3, 4])
        );
        assert_eq!(
            merge(&Some(left.clone()), &None, true),
            Some(vec![1, 2, 3, 4])
        );

        assert_eq!(
            merge(&None, &Some(right.clone()), false),
            Some(vec![5, 6, 7])
        );
        assert_eq!(
            merge(&None, &Some(right.clone()), true),
            Some(vec![5, 6, 7])
        );

        assert_eq!(
            merge(&Some(left.clone()), &empty.clone(), false),
            Some(vec![1, 2, 3, 4])
        );
        assert_eq!(merge(&Some(left.clone()), &empty.clone(), true), None);

        assert_eq!(
            merge(&empty.clone(), &Some(right.clone()), false),
            Some(vec![5, 6, 7])
        );
        assert_eq!(
            merge(&empty.clone(), &Some(right.clone()), true),
            Some(vec![5, 6, 7])
        );
    }

    #[test]
    fn test_merge_returns_none() {
        let none: Option<Vec<i32>> = None;
        let empty: Option<Vec<i32>> = Some(Vec::new());

        assert_eq!(merge(&none.clone(), &none.clone(), true), None);
        assert_eq!(merge(&none.clone(), &none.clone(), false), None);

        assert_eq!(merge(&none.clone(), &empty.clone(), true), None);
        assert_eq!(merge(&none.clone(), &empty.clone(), false), None);

        assert_eq!(merge(&empty.clone(), &empty.clone(), true), None);
        assert_eq!(merge(&empty.clone(), &empty.clone(), false), None);

        assert_eq!(merge(&empty.clone(), &none.clone(), true), None);
        assert_eq!(merge(&empty.clone(), &none.clone(), false), None);
    }
}
