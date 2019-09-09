use std::path::PathBuf;

use clap::{
    App,
    SubCommand,
};
use shiplift::BuildOptions;
use super::docker;
use super::aspects;

pub struct ContainerManager {
    path: PathBuf,
    image_name: String,
    image_tag: String,
    dependencies: Vec<Box<ContainerManager>>,
    aspects: Vec<Box<dyn aspects::ContainerAspect>>,
    args: Vec<String>,
}

pub fn new_container_manager(
    path: PathBuf,
    image_name: String,
    image_tag: String,
    dependencies: Vec<Box<ContainerManager>>,
    aspects: Vec<Box<dyn aspects::ContainerAspect>>,
    args: Vec<String>,
) -> ContainerManager {
    ContainerManager{
        path: path,
        image_name: image_name,
        image_tag: image_tag,
        dependencies: dependencies,
        aspects: aspects,
        args: args,
    }
}

impl ContainerManager {
    fn image(&self) -> String {
        String::from(format!("{}:{}", self.image_name, &self.image_tag))
    }

    fn run(&self, name: &str) -> Result<(), ()> {
        let mut args: Vec<String> = vec![
            "--rm",
            "--name", name

        ].into_iter()
            .map(String::from)
            .collect();

        for aspect in &self.aspects {
            args.extend(aspect.run_args());
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
        let opts = &BuildOptions::builder(self.path.to_str().unwrap())
            .tag(self.image())
            .build();
        docker::build(opts);
        Ok(())
    }

    pub fn execute(&self, name: &str) {
        let matches = App::new(name)
            .version("0.0")
            .subcommand(SubCommand::with_name("run"))
            .subcommand(SubCommand::with_name("build"))
            .get_matches();

        match matches.subcommand() {
            ("run", _) => self.run(&name).unwrap(),
            ("build", _) => self.build().unwrap(),
            (_, _) => {
                self.build().unwrap();
                self.run(&name).unwrap();
            }
        }
    }
}
