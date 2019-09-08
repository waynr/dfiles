use std::path::{
    PathBuf,
};

use dfiles::containermanager::{
    NewContainerManager,
    ContainerAspects
};

fn main() {
    let cm = NewContainerManager(
        PathBuf::new(),
        String::from("waynr/chrome"),
        String::from("dev"),
        Vec::new(),
        Vec::new(),
    );
}
