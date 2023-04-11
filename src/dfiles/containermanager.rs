use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

use clap::{Arg, ArgAction, ArgMatches, Command};
use dockworker::{ContainerBuildOptions, Docker};
use dyn_clone;
use serde::Deserialize;
use serde_json::from_str;
use tar::{Builder, Header};
use tempfile::NamedTempFile;

use super::aspects;
use super::config;
use super::docker;
use super::entrypoint;
use super::error::{Error, Result};
use super::logging;

#[derive(Deserialize, Debug)]
struct BuildOutput {
    stream: String,
}

pub struct ContainerManager {
    name: String,
    tags: Vec<String>,
    container_paths: Vec<String>,
    aspects: Vec<Box<dyn aspects::ContainerAspect>>,
    args: Vec<String>,
    tempdir: tempfile::TempDir,
}

impl ContainerManager {
    pub fn default(
        name: String,
        tags: Vec<String>,
        container_paths: Vec<String>,
        aspects: Vec<Box<dyn aspects::ContainerAspect>>,
        args: Vec<String>,
    ) -> Result<ContainerManager> {
        let tempdir = tempfile::Builder::new()
            .prefix(&format!("dfiles-{}-{}-", name, std::process::id()))
            .tempdir()?;
        Ok(ContainerManager {
            name,
            tags,
            container_paths,
            aspects,
            args,
            tempdir,
        })
    }
    pub fn default_debian(
        name: String,
        tags: Vec<String>,
        container_paths: Vec<String>,
        mut aspects: Vec<Box<dyn aspects::ContainerAspect>>,
        args: Vec<String>,
        version: Option<String>,
    ) -> Result<ContainerManager> {
        let aspect = match version {
            None => String::from("buster"),
            Some(s) => s,
        };
        aspects.insert(0, Box::new(Debian { version: aspect }));
        Self::default(name, tags, container_paths, aspects, args)
    }

    pub fn default_ubuntu(
        name: String,
        tags: Vec<String>,
        container_paths: Vec<String>,
        mut aspects: Vec<Box<dyn aspects::ContainerAspect>>,
        args: Vec<String>,
        version: Option<String>,
    ) -> Result<ContainerManager> {
        let aspect = match version {
            None => String::from("20.04"),
            Some(s) => s,
        };
        aspects.insert(0, Box::new(Ubuntu { version: aspect }));
        Self::default(name, tags, container_paths, aspects, args)
    }

    fn image(&self) -> String {
        self.tags[0].clone()
    }

    fn run(&self, matches: &ArgMatches) -> Result<()> {
        let mut args: Vec<String> = vec!["--rm"].into_iter().map(String::from).collect();

        log::debug!("active dfiles aspects:");
        for aspect in &self.aspects {
            log::debug!("{:}", aspect);
            args.extend(aspect.run_args(Some(matches))?);
        }

        let ep_args = entrypoint::setup(self.tempdir.path(), &self.aspects)?;
        args.extend(ep_args);
        args.push(self.image());
        args.extend_from_slice(&self.args);
        docker::run(args);
        Ok(())
    }

    fn cmd(&self, matches: &ArgMatches) -> Result<()> {
        let mut args: Vec<String> = vec!["-it", "--rm"].into_iter().map(String::from).collect();

        log::debug!("active dfiles aspects:");
        for aspect in &self.aspects {
            log::debug!("{:}", aspect);
            args.extend(aspect.run_args(Some(matches))?);
        }

        let command: Vec<String> = matches
            .try_get_many::<String>("command")?
            .map_or(vec!["/bin/bash".to_string()], |values_ref| {
                values_ref.map(String::from).collect()
            });

        let ep_args = entrypoint::setup(self.tempdir.path(), &self.aspects)?;
        args.extend(ep_args);
        args.push(self.image());
        args.extend_from_slice(command.as_slice());

        docker::run(args);
        Ok(())
    }

    fn build(&self) -> Result<()> {
        let mut tar_file = NamedTempFile::new()?;
        self.generate_archive_impl(tar_file.as_file_mut())?;

        let docker = Docker::connect_with_defaults()?;
        let options = ContainerBuildOptions {
            dockerfile: "Dockerfile".into(),
            t: self.tags.clone(),
            ..ContainerBuildOptions::default()
        };

        let res = docker.build_image(options, tar_file.path())?;
        BufReader::new(res)
            .lines()
            .filter_map(std::result::Result::ok)
            .map(|l| from_str::<BuildOutput>(&l))
            .filter_map(std::result::Result::ok)
            .for_each(|bo: BuildOutput| print!("{}", bo.stream));
        Ok(())
    }

    fn generate_archive_impl(&self, f: &mut std::fs::File) -> Result<()> {
        let mut a = Builder::new(f);

        let mut contents: BTreeMap<u8, String> = BTreeMap::new();
        for aspect in &self.aspects {
            let dockerfile_snippets = aspect.dockerfile_snippets();
            for snippet in dockerfile_snippets {
                contents
                    .entry(snippet.order)
                    .and_modify(|e| {
                        e.push('\n');
                        e.push_str(snippet.content.as_str());
                    })
                    .or_insert(snippet.content);
            }
            for file in aspect.container_files() {
                add_file_to_archive(&mut a, &file.container_path, &file.contents)?;
            }
        }

        let mut dockerfile_contents = String::new();

        for content in contents.values() {
            dockerfile_contents.push_str(content.as_str());
            dockerfile_contents.push('\n');
            dockerfile_contents.push('\n');
        }

        add_file_to_archive(&mut a, "Dockerfile", dockerfile_contents.as_bytes())?;

        Ok(())
    }

    fn generate_archive(&self) -> Result<()> {
        let mut tar_file = File::create(format!("{}.tar", self.name))?;
        self.generate_archive_impl(&mut tar_file)
    }

    /// Takes configuration options for the dfiles binary and saves them to be loaded at build or
    /// run time.
    ///
    /// dfiles strives to provide a configurable framework for building and running GUI containers.
    /// to achieve this configurability, we allow dynamic Aspects to be loaded from configuration
    /// files. Those configuration files can be hand-written but we also provide a `config`
    /// subcommand.
    ///
    /// ```bash
    /// $ firefox config --mount <hostpath>:<containerpath>
    /// ```
    fn config(&self, matches: &ArgMatches) -> Result<()> {
        let cfg = config::Config::try_from(matches)?;

        let profile: Option<&str> = matches
            .try_get_one::<String>("profile")?
            .map(|x| x.as_str());

        cfg.save(Some(&self.name), profile)
    }

    fn load_config(&mut self, matches: &ArgMatches) -> Result<()> {
        let profile: Option<&str> = matches
            .try_get_one::<String>("profile")?
            .map(|x| x.as_str());

        let cfg = config::Config::load(&self.name, profile)?;

        let cli_cfg = config::Config::try_from(matches)?;

        self.aspects
            .extend(cfg.merge(&cli_cfg, false).get_aspects());
        Ok(())
    }

    pub fn execute(&mut self) -> Result<()> {
        let mut run = Command::new("run").about("run app in container");
        let mut cmd = Command::new("cmd").about("run specified command in container");
        let mut build = Command::new("build").about("build app container");
        let mut config = Command::new("config").about("configure app container settings");
        let generate_archive =
            Command::new("generate-archive").about("generate archive used to build container");

        let mut app = Command::new(&self.name).version("0.0").arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .value_parser(clap::value_parser!(u8))
                .action(ArgAction::Count),
        );

        self.aspects.insert(
            0,
            Box::new(aspects::Profile {
                name: self.name.clone(),
                container_paths: self.container_paths.clone(),
            }),
        );

        for arg in &config::cli_args() {
            run = run.arg(arg);
            cmd = cmd.arg(arg);
            config = config.arg(arg);
        }

        cmd = cmd.arg(
            Arg::new("command")
                .action(ArgAction::Set)
                .last(true)
                .required(true)
                .help("command to run instead of default"),
        );

        let cloned = dyn_clone::clone_box(&self.aspects);
        for aspect in cloned.iter() {
            for arg in aspect.config_args() {
                run = run.arg(arg);
            }
            for arg in aspect.config_args() {
                cmd = cmd.arg(arg);
            }
            for arg in aspect.cli_build_args() {
                build = build.arg(arg);
            }
            for arg in aspect.config_args() {
                config = config.arg(arg);
            }
        }

        app = app
            .subcommand(run)
            .subcommand(cmd)
            .subcommand(build)
            .subcommand(config)
            .subcommand(generate_archive);

        let matches = app.get_matches_mut();

        let level = matches.get_one("verbose").unwrap_or(&0u8);
        logging::setup(*level)?;

        match matches.subcommand() {
            Some((cmd, subm)) => {
                self.load_config(subm)?;
                match cmd {
                    "run" => self.run(subm),
                    "cmd" => self.cmd(subm),
                    "build" => self.build(),
                    "config" => self.config(subm),
                    "generate-archive" => self.generate_archive(),
                    &_ => Ok(println!("{}", app.render_usage())),
                }
            }
            None => Ok(println!("{}", app.render_usage())),
        }
    }
}

fn add_file_to_archive<W: Write>(b: &mut Builder<W>, name: &str, contents: &[u8]) -> Result<()> {
    let mut header = Header::new_gnu();
    header
        .set_path(name)
        .map_err(|e| Error::FailedToAddFileToArchive { source: e })?;
    header.set_size(contents.len() as u64);
    header.set_cksum();
    b.append(&header, contents)
        .map_err(|e| Error::FailedToAddFileToArchive { source: e })
}

#[derive(Clone)]
struct Debian {
    pub version: String,
}

impl aspects::ContainerAspect for Debian {
    fn name(&self) -> String {
        String::from("Debian")
    }
    fn dockerfile_snippets(&self) -> Vec<aspects::DockerfileSnippet> {
        vec![
            aspects::DockerfileSnippet {
                order: 00,
                content: format!("FROM debian:{}", self.version),
            },
            aspects::DockerfileSnippet {
                order: 3,
                content: String::from(
                    r#"# Useful language packs
RUN apt-get update && apt-get install -y --no-install-recommends \
  fonts-arphic-bkai00mp \
  fonts-arphic-bsmi00lp \
  fonts-arphic-gbsn00lp \
  fonts-arphic-gbsn00lp \
  \
  && rm -rf /var/lib/apt/lists/* \
  && rm -rf /src/*.deb"#,
                ),
            },
            aspects::DockerfileSnippet {
                order: 2,
                content: String::from(
                    r#"RUN apt-get update && apt-get install -y \
    --no-install-recommends \
    apt-utils \
    apt-transport-https \
    apt \
    bzip2 \
    ca-certificates \
    curl \
    debian-goodies \
    dirmngr \
    gnupg \
    keychain \
    lsb-release \
    locales \
    lsof \
    procps \
    sudo \
  && apt-get purge --autoremove \
  && rm -rf /var/lib/apt/lists/* \
  && rm -rf /src/*.deb "#,
                ),
            },
        ]
    }
}

#[derive(Clone)]
struct Ubuntu {
    pub version: String,
}

impl aspects::ContainerAspect for Ubuntu {
    fn name(&self) -> String {
        String::from("Ubuntu")
    }
    fn dockerfile_snippets(&self) -> Vec<aspects::DockerfileSnippet> {
        vec![
            aspects::DockerfileSnippet {
                order: 00,
                content: format!("FROM ubuntu:{}", self.version),
            },
            aspects::DockerfileSnippet {
                order: 3,
                content: String::from(
                    r#"# Useful language packs
RUN apt-get update && apt-get install -y --no-install-recommends \
  fonts-arphic-bkai00mp \
  fonts-arphic-bsmi00lp \
  fonts-arphic-gbsn00lp \
  fonts-arphic-gbsn00lp \
  \
  && rm -rf /var/lib/apt/lists/* \
  && rm -rf /src/*.deb"#,
                ),
            },
            aspects::DockerfileSnippet {
                order: 2,
                content: String::from(
                    r#"RUN apt-get update && apt-get install -y \
    --no-install-recommends \
    apt-utils \
    apt-transport-https \
    apt \
    bzip2 \
    ca-certificates \
    curl \
    debian-goodies \
    dirmngr \
    gnupg \
    keychain \
    lsb-release \
    locales \
    lsof \
    procps \
  && apt-get purge --autoremove \
  && rm -rf /var/lib/apt/lists/* \
  && rm -rf /src/*.deb "#,
                ),
            },
        ]
    }
}
