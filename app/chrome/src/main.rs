use std::path::{
    PathBuf,
};

use dfiles::containermanager::{
    NewContainerManager,
    ContainerAspects
};

fn main() {
    let chrome_dir = PathBuf::from("/home/wayne/projects/dockerfiles/chrome");

    let mgr = NewContainerManager(
        chrome_dir,
        String::from("waynr/chrome"),
        String::from("v0"),
        Vec::new(),
        Vec::new(),
    );

    mgr.execute(String::from("chrome"));
}
