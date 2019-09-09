use std::path::{
    PathBuf,
};
use std::{
    env,
    fs,
};

use dfiles::aspects;
use dfiles::containermanager::{
    new_container_manager,
};

struct Chrome {}
impl aspects::ContainerAspect for Chrome {
    fn run_args(&self) -> Vec<String> {
        let home = env::var("HOME")
            .expect("HOME must be set");

        vec![
            "--cap-add", "SYS_ADMIN",
            "--cpu-shares", "512",
            "--memory", "3072mb",
            "-v", "/dev/shm:/dev/shm",

            "-v", format!("{}/.config/google-chrome:/data", home).as_str(),
            "-v", format!("{}/downloads:/home/wayne/Downloads", home).as_str(),

            "--name", "chrome",
        ].into_iter()
            .map(String::from)
            .collect()
    }
}

fn main() {
    let chrome_dir = PathBuf::from("/home/wayne/projects/dockerfiles/chrome");

    let mgr = new_container_manager(
        chrome_dir,
        String::from("waynr/chrome"),
        String::from("v0"),
        Vec::new(),
        vec![
            Box::new(Chrome{}),
            Box::new(aspects::PulseAudio{}),
            Box::new(aspects::X11{}),
            Box::new(aspects::Video{}),
            Box::new(aspects::DBus{}),
            Box::new(aspects::NetHost{}),
            Box::new(aspects::SysAdmin{}),
        ],
    );

    mgr.execute(String::from("chrome"));
}
