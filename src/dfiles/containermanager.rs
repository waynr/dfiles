pub trait ContainerManager {
    fn dependencies(&self) -> &Vec<Box<dyn ContainerManager>>;
    fn build(&self) -> Result<(), ()>;
    fn run(&self) -> Result<(), ()>;
    fn image(&self) -> String;
}

pub fn build_deps<T: ContainerManager>(cm: &T) {
    let dependencies = cm.dependencies();
    for dep in dependencies {
        dep.build().unwrap();
    }
}
