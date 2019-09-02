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

