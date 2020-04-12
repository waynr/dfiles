use std::collections::HashMap;
use std::io::{BufRead, BufReader};

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

    fn build(&self) -> Result<(), ()> {
        self.build_deps();
        let tar_file = NamedTempFile::new().unwrap();
        let mut a = Builder::new(&tar_file);

        for (name, bs) in &self.context {
            let mut header = Header::new_gnu();
            header.set_path(name).unwrap();
            header.set_size(bs.len() as u64);
            header.set_cksum();
            a.append(&header, bs.as_bytes()).unwrap();
        }

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

    fn generate_archive(&self) -> Result<(), ()> {
        Ok(())
    }

    pub fn execute(&self, name: &str) {
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
