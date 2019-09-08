use std::path::{
    PathBuf,
};

pub trait Buildable {
    fn dependencies(&self) -> &Vec<Box<dyn Buildable>>;
    fn build(&self) -> Result<(), ()>;

    fn build_deps(&self) {
        let dependencies = self.dependencies();
        for dep in dependencies {
            dep.build().unwrap();
        }
    }
}

pub trait Runnable : Buildable {
    fn run(&self) -> Result<(), ()>;
}


pub struct PulseAudio {}
pub struct X11 {}
pub struct Video {}
pub struct DBus {}

pub enum ContainerAspects {
    PulseAudio,
    X11,
    Video,
    DBus,
}

pub struct ContainerManager {
    path: PathBuf,
    image_name: String,
    image_tag: String,
    dependencies: Vec<Box<dyn Buildable>>,
    aspects: Vec<ContainerAspects>,
}

pub fn NewContainerManager(
    path: PathBuf,
    image_name: String,
    image_tag: String,
    dependencies: Vec<Box<dyn Buildable>>,
    aspects: Vec<ContainerAspects>,
) -> ContainerManager {
    ContainerManager{
        path: path,
        image_name: image_name,
        image_tag: image_tag,
        aspects: aspects,
        dependencies: dependencies,
    }
}
