use std::collections::BTreeMap;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

use clap::{App, ArgMatches, SubCommand};
use dockworker::{ContainerBuildOptions, Docker};
use serde::Deserialize;
use serde_json::from_str;
use tar::{Builder, Header};
use tempfile::NamedTempFile;

use super::aspects;
use super::docker;

#[derive(Deserialize, Debug)]
struct BuildOutput {
    stream: String,
}

pub struct ContainerManager {
    context: HashMap<String, String>,
    tags: Vec<String>,
    dependencies: Vec<Box<ContainerManager>>,
    aspects: Vec<Box<dyn aspects::ContainerAspect>>,
    args: Vec<String>,
}

pub fn new_container_manager(
    context: HashMap<String, String>,
    tags: Vec<String>,
    dependencies: Vec<Box<ContainerManager>>,
    aspects: Vec<Box<dyn aspects::ContainerAspect>>,
    args: Vec<String>,
) -> ContainerManager {
    ContainerManager {
        context: context,
        tags: tags,
        dependencies: dependencies,
        aspects: aspects,
        args: args,
    }
}

pub fn default_debian_container_manager(
    context: HashMap<String, String>,
    tags: Vec<String>,
    dependencies: Vec<Box<ContainerManager>>,
    mut aspects: Vec<Box<dyn aspects::ContainerAspect>>,
    args: Vec<String>,
) -> ContainerManager {
    aspects.insert(0, Box::new(Debian {}));
    ContainerManager {
        context: context,
        tags: tags,
        dependencies: dependencies,
        aspects: aspects,
        args: args,
    }
}

pub fn noop_container_manager(
    context: HashMap<String, String>,
    tags: Vec<String>,
) -> ContainerManager {
    ContainerManager {
        context: context,
        tags: tags,
        args: Vec::new(),
        aspects: Vec::new(),
        dependencies: Vec::new(),
    }
}

impl ContainerManager {
    fn image(&self) -> String {
        self.tags[0].clone()
    }

    fn run<'a>(&self, matches: &'a ArgMatches<'a>) -> Result<(), ()> {
        let mut args: Vec<String> = vec!["--rm"].into_iter().map(String::from).collect();

        for aspect in &self.aspects {
            println!("{:}", aspect);
            args.extend(aspect.run_args(Some(&matches)));
        }

        args.push(self.image().to_string());
        args.extend_from_slice(&self.args);
        docker::run(args);
        Ok(())
    }

    fn build_deps(&self) {
        for dep in &self.dependencies {
            dep.build().unwrap();
        }
    }

    fn build(&self) -> Result<(), Box<dyn Error>> {
        self.build_deps();
        let mut tar_file = NamedTempFile::new().unwrap();
        self.generate_archive_impl(&mut tar_file.as_file_mut())?;

        let docker = Docker::connect_with_defaults().unwrap();
        let options = ContainerBuildOptions {
            dockerfile: "Dockerfile".into(),
            t: self.tags.clone(),
            ..ContainerBuildOptions::default()
        };

        let res = docker.build_image(options, tar_file.path()).unwrap();
        BufReader::new(res)
            .lines()
            .filter_map(Result::ok)
            .map(|l| from_str::<BuildOutput>(&l))
            .filter_map(Result::ok)
            .for_each(|bo: BuildOutput| print!("{}", bo.stream));
        Ok(())
    }

    fn generate_archive_impl(&self, f: &mut std::fs::File) -> Result<(), Box<dyn Error>> {
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

        add_file_to_archive(&mut a, "Dockerfile", &dockerfile_contents)?;

        for (name, bs) in &self.context {
            add_file_to_archive(&mut a, name, bs)?;
        }
        Ok(())
    }

    fn generate_archive(&self) -> Result<(), Box<dyn Error>> {
        let mut tar_file = File::create("whatever.tar")?;
        self.generate_archive_impl(&mut tar_file)
    }

    pub fn execute(&mut self, name: &str) {
        let mut run = SubCommand::with_name("run").about("run app in container");
        let mut build = SubCommand::with_name("build").about("build app container");
        let generate_archive = SubCommand::with_name("generate-archive")
            .about("generate archvie used to build container");

        let mut app = App::new(name).version("0.0");

        for aspect in &self.aspects {
            for arg in aspect.cli_run_args() {
                run = run.arg(arg);
            }
            for arg in aspect.cli_build_args() {
                build = build.arg(arg);
            }
        }

        app = app
            .subcommand(run)
            .subcommand(build)
            .subcommand(generate_archive);

        let matches = app.get_matches();

        match matches.subcommand() {
            ("run", Some(subm)) => self.run(&subm).unwrap(),
            ("build", _) => self.build().unwrap(),
            ("generate-archive", _) => self.generate_archive().unwrap(),
            (_, _) => println!("{}", matches.usage()),
        }
    }
}

fn add_file_to_archive<W: Write>(
    b: &mut Builder<W>,
    name: &str,
    contents: &str,
) -> Result<(), std::io::Error> {
    let mut header = Header::new_gnu();
    header.set_path(name).unwrap();
    header.set_size(contents.len() as u64);
    header.set_cksum();
    b.append(&header, contents.as_bytes())
}

struct Debian {}

impl aspects::ContainerAspect for Debian {
    fn name(&self) -> String {
        String::from("Debian")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        Vec::new()
    }
    fn dockerfile_snippets(&self) -> Vec<aspects::DockerfileSnippet> {
        vec![
            aspects::DockerfileSnippet {
                order: 00,
                content: String::from("FROM debian:buster"),
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
