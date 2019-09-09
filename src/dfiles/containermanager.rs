use std::path::{
    PathBuf,
    Path,
};
use std::{
    env,
    fs,
};


use clap::{
    App,
    Arg,
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
    aspects: Vec<Box<aspects::ContainerAspect>>,
}

pub fn new_container_manager(
    path: PathBuf,
    image_name: String,
    image_tag: String,
    dependencies: Vec<Box<ContainerManager>>,
    aspects: Vec<Box<aspects::ContainerAspect>>,
) -> ContainerManager {
    ContainerManager{
        path: path,
        image_name: image_name,
        image_tag: image_tag,
        aspects: aspects,
        dependencies: dependencies,
    }
}

impl ContainerManager {
    fn image(&self) -> String {
        String::from(format!("{}:{}", self.image_name, &self.image_tag))
    }

    fn run(&self) -> Result<(), ()> {
        let home = env::var("HOME")
            .expect("HOME must be set");

        let mut args: Vec<String> = vec![
            "-i", "-t", "--rm",

            "--cpu-shares", "512",
            "--memory", "3072mb",
            "-v", "/dev/shm:/dev/shm",

            "-v", format!("{}/.config/google-chrome:/data", home).as_str(),
            "-v", format!("{}/downloads:/home/wayne/Downloads", home).as_str(),
            "--name", "chrome",
        ].into_iter()
            .map(String::from)
            .collect();

        for aspect in &self.aspects {
            args.extend(aspect.run_args());
        }

        args.push(self.image().to_string());
        args.push(String::from("google-chrome"));
        args.push(String::from("--user-data-dir=/data"));
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

    pub fn execute(&self, name: String) {
        let matches = App::new(name)
            .version("0.0")
            .subcommand(SubCommand::with_name("run"))
            .subcommand(SubCommand::with_name("build"))
            .get_matches();

        match matches.subcommand() {
            ("run", _) => self.run().unwrap(),
            ("build", _) => self.build().unwrap(),
            (_, _) => {
                self.build().unwrap();
                self.run().unwrap();
            }
        }
    }
}
