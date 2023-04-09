use std::convert::TryFrom;
use std::fs;
use std::io::Write;

use clap::{Arg, ArgMatches};
use serde::{Deserialize, Serialize};

use super::aspects;
use super::dirs;
use super::error::{Error, Result};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub mounts: Option<Vec<aspects::Mount>>,
    pub timezone: Option<aspects::Timezone>,
    pub memory: Option<aspects::Memory>,
    pub cpu_shares: Option<aspects::CPUShares>,
    pub network: Option<aspects::Network>,
    pub locale: Option<aspects::Locale>,
}

impl Config {
    pub fn empty() -> Config {
        Config {
            mounts: None,
            timezone: None,
            memory: None,
            cpu_shares: None,
            network: None,
            locale: None,
        }
    }

    pub fn save(&self, application: Option<&str>, profile: Option<&str>) -> Result<()> {
        let existing_config = Config::load_layer(application, profile)?;
        let merged = existing_config.merge(self, true);

        let config_dir = dirs::get_config_dir(application, profile)?;
        fs::create_dir_all(&config_dir)?;

        let path = config_dir.join("config.yaml");
        let mut config_file = fs::File::create(path)?;

        let s = serde_yaml::to_string(&merged).map_err(|_| Error::FailedToSaveConfig)?;
        config_file.write_all(&s.into_bytes())?;

        Ok(())
    }

    /// Loads a single config file specified by the combination of application and profile options;
    /// if both are none, then loads the global config.
    fn load_layer(application: Option<&str>, profile: Option<&str>) -> Result<Config> {
        let config_dir = dirs::get_config_dir(application, profile)?;
        let yaml_file = config_dir.join("config.yaml");

        let mut cfg = Config::empty();

        if yaml_file.exists() {
            let yaml = fs::read_to_string(yaml_file)?;
            cfg = serde_yaml::from_str(&yaml).map_err(|_| Error::FailedToLoadConfig)?;
        }

        Ok(cfg)
    }

    pub fn load(application: &str, profile: Option<&str>) -> Result<Config> {
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

    /// Merge aspects from the given Config into a copy of the current, return a new Config.
    pub fn merge(&self, other: &Config, overwrite: bool) -> Config {
        let mut cfg = (*self).clone();

        cfg.mounts = merge(&self.mounts, &other.mounts, overwrite);

        if let Some(v) = &other.timezone {
            cfg.timezone = Some(v.clone());
        }

        if let Some(v) = &other.memory {
            cfg.memory = Some(v.clone());
        }

        if let Some(v) = &other.cpu_shares {
            cfg.cpu_shares = Some(v.clone());
        }

        if let Some(v) = &other.network {
            cfg.network = Some(v.clone());
        }

        if let Some(v) = &other.locale {
            cfg.locale = Some(v.clone());
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

        if let Some(memory) = &self.memory {
            aspects.push(Box::new(memory.clone()));
        }

        if let Some(cpu_shares) = &self.cpu_shares {
            aspects.push(Box::new(cpu_shares.clone()));
        }

        if let Some(network) = &self.network {
            aspects.push(Box::new(network.clone()));
        }

        if let Some(locale) = &self.locale {
            aspects.push(Box::new(locale.clone()));
        }

        aspects
    }
}

impl TryFrom<&ArgMatches<'_>> for Config {
    type Error = Error;
    fn try_from(matches: &ArgMatches) -> Result<Self> {
        let mut cfg = Config::empty();

        if let Some(vs) = matches.values_of("mount") {
            let mut mounts: Vec<aspects::Mount> = Vec::new();
            for v in vs {
                mounts.push(aspects::Mount::try_from(v)?);
            }
            cfg.mounts = Some(mounts);
        }

        if let Some(tz) = matches.value_of("timezone") {
            cfg.timezone = Some(aspects::Timezone::try_from(tz)?);
        }

        if let Some(memory) = matches.value_of("memory") {
            cfg.memory = Some(aspects::Memory::try_from(memory)?);
        }

        if let Some(cpu_shares) = matches.value_of("cpu-shares") {
            cfg.cpu_shares = Some(aspects::CPUShares::try_from(cpu_shares)?);
        }

        if let Some(network) = matches.value_of("network") {
            cfg.network = Some(aspects::Network::try_from(network)?);
        }

        if let Some(locale) = matches.value_of("locale") {
            cfg.locale = Some(aspects::Locale::try_from(locale)?);
        }

        Ok(cfg)
    }
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

pub fn cli_args<'a, 'b>() -> Vec<Arg<'a, 'b>> {
    vec![
        Arg::with_name("mount")
            .short("m")
            .long("mount")
            .multiple(true)
            .takes_value(true)
            .help("specify a local path to be mapped into the container filesystem at runtime"),
        Arg::with_name("timezone")
            .short("t")
            .long("timezone")
            .takes_value(true)
            .help("specify the timezone to be built into the container image"),
        Arg::with_name("memory")
            .long("memory")
            .takes_value(true)
            .help("specify the runtime memory resource limit"),
        Arg::with_name("cpu-shares")
            .long("cpu-shares")
            .takes_value(true)
            .help("specify the runtime proportion of cpu cycles for the container"),
        Arg::with_name("network")
            .long("network")
            .takes_value(true)
            .help("specify the runtime network mode for the container (default: bridge)"),
        Arg::with_name("locale")
            .long("locale")
            .takes_value(true)
            .help("specify the locale in the form <language>_<territory>.<codeset> for the container (default: en_US.UTF8)"),
    ]
}

#[cfg(test)]
mod merge_should {
    use super::*;

    #[test]
    fn return_some() {
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
            merge(&Some(left.clone()), &empty, false),
            Some(vec![1, 2, 3, 4])
        );
        assert_eq!(merge(&Some(left), &empty, true), None);

        assert_eq!(
            merge(&empty, &Some(right.clone()), false),
            Some(vec![5, 6, 7])
        );
        assert_eq!(merge(&empty, &Some(right), true), Some(vec![5, 6, 7]));
    }

    #[test]
    fn return_none() {
        let none: Option<Vec<i32>> = None;
        let empty: Option<Vec<i32>> = Some(Vec::new());

        assert_eq!(merge(&none, &none, true), None);
        assert_eq!(merge(&none, &none, false), None);

        assert_eq!(merge(&none, &empty, true), None);
        assert_eq!(merge(&none, &empty, false), None);

        assert_eq!(merge(&empty, &empty, true), None);
        assert_eq!(merge(&empty, &empty, false), None);

        assert_eq!(merge(&empty, &none, true), None);
        assert_eq!(merge(&empty, &none, false), None);
    }
}
