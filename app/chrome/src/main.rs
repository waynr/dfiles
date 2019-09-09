use std::path::{
    PathBuf,
};

use dfiles::containermanager::{
    new_container_manager,
};
use dfiles::aspects;

fn main() {
    let chrome_dir = PathBuf::from("/home/wayne/projects/dockerfiles/chrome");

    let mgr = new_container_manager(
        chrome_dir,
        String::from("waynr/chrome"),
        String::from("v0"),
        Vec::new(),
        vec![ Box::new(aspects::PulseAudio{}),
            Box::new(aspects::X11{}),
            Box::new(aspects::Video{}),
            Box::new(aspects::DBus{}),
            Box::new(aspects::NetHost{}),
            Box::new(aspects::SysAdmin{}),
        ],
    );

    mgr.execute(String::from("chrome"));
}
