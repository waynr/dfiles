use std::path::PathBuf;

use shiplift::BuildOptions;
use shiplift::ContainerOptions;

use super::super::containermanager::ContainerManager;
use super::super::containermanager::build_deps;
use super::super::docker;

pub struct Chrome {
    path_buf: PathBuf,
    image_name: String,
    image_tag: String,
    dependencies: Vec<Box<dyn ContainerManager>>
}

impl ContainerManager for Chrome {
    fn image(&self) -> String {
        String::from(format!("{}:{}", self.image_name, &self.image_tag))
    }

    fn build(&self) -> Result<(), ()> {
        build_deps(self);
        let opts = &BuildOptions::builder(self.path_buf.to_str().unwrap())
            .tag(self.image())
            .build();
        docker::build(opts);
        Ok(())
    }

    fn run(&self) -> Result<(), ()> {
        let opts = &ContainerOptions::builder(self.image().as_str())
            .build();
        docker::run(opts);
        Ok(())
    }

    fn dependencies(&self) -> &Vec<Box<dyn ContainerManager>> {
        &self.dependencies
    }
}

pub fn new(p: PathBuf) -> Chrome {

    Chrome{
        image_name: String::from("waynr/chrome"),
        image_tag: String::from("v0"),
        path_buf: p,
        dependencies: Vec::new(),
    }
}
