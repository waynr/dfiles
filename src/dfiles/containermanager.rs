use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use super::aspects;
use super::docker;
use clap::{App, Arg, ArgMatches, SubCommand};
use dockworker::{ContainerBuildOptions, Docker};

pub struct ContainerManager {
    context_tarball: PathBuf,
    tags: Vec<String>,
    dependencies: Vec<Box<ContainerManager>>,
    aspects: Vec<Box<dyn aspects::ContainerAspect>>,
    args: Vec<String>,
}

pub fn new_container_manager(
    context_tarball: PathBuf,
    tags: Vec<String>,
    dependencies: Vec<Box<ContainerManager>>,
    aspects: Vec<Box<dyn aspects::ContainerAspect>>,
    args: Vec<String>,
) -> ContainerManager {
    ContainerManager {
        context_tarball: context_tarball,
        tags: tags,
        dependencies: dependencies,
        aspects: aspects,
        args: args,
    }
}

impl ContainerManager {
    fn image(&self) -> String {
        self.tags[0].clone()
    }

    fn run<'a>(&self, matches: &'a ArgMatches<'a>) -> Result<(), ()> {
        let mut name = "dfiles";

        if let Some(c) = matches.value_of("container_name") {
            name = c
        }

        let mut args: Vec<String> = vec!["--rm", "--name", name]
            .into_iter()
            .map(String::from)
            .collect();

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
        let docker = Docker::connect_with_defaults().unwrap();
        let options = ContainerBuildOptions {
            dockerfile: "Dockerfile".into(),
            t: self.tags.clone(),
            ..ContainerBuildOptions::default()
        };
        let res = docker.build_image(options, &self.context_tarball).unwrap();
        for line in BufReader::new(res).lines() {
            let buf = line.unwrap();
            println!("{}", &buf);
        }
        Ok(())
    }

    pub fn execute(&self, name: &str) {
        let mut run = SubCommand::with_name("run").about("run app in container");
        let mut build = SubCommand::with_name("build").about("build app container");

        let mut app = App::new(name).version("0.0");

        for aspect in &self.aspects {
            for arg in aspect.cli_run_args() {
                println!("meow");
                run = run.arg(arg);
            }
            for arg in aspect.cli_build_args() {
                build = build.arg(arg);
            }
        }

        run = run.arg(
            Arg::with_name("container_name")
                .short("n")
                .long("name")
                .help("specify the name of the container to be run")
                .global(true)
                .takes_value(true),
        );

        app = app.subcommand(run).subcommand(build);

        let matches = app.get_matches();

        match matches.subcommand() {
            ("run", Some(subm)) => self.run(&subm).unwrap(),
            ("build", _) => self.build().unwrap(),
            (_, _) => println!("{}", matches.usage()),
        }
    }
}
