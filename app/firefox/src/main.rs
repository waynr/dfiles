use std::path::{
    PathBuf,
};
use std::{
    env,
};

use dfiles::aspects;
use dfiles::containermanager::{
    new_container_manager,
};

struct Firefox {}
impl aspects::ContainerAspect for Firefox {
    fn run_args(&self) -> Vec<String> {
        let home = env::var("HOME")
            .expect("HOME must be set");
        let profile = "digitalocean";

        vec![
            "--cpu-shares", "512",
            "--memory", "3072mb",
            "-v", "/dev/shm:/dev/shm",

            "-v", format!("{h}/.mozilla/firefox/{p}:{h}/.mozilla/firefox/profile", h=home, p=profile).as_str(),
            "-v", format!("{}/downloads:/home/wayne/Downloads", home).as_str(),

            "--name", "firefox",
        ].into_iter()
            .map(String::from)
            .collect()
    }
}

fn main() {
    let home = env::var("HOME")
        .expect("HOME must be set");
    let firefox_dir = PathBuf::from("/home/wayne/projects/dockerfiles/firefox");

    let mgr = new_container_manager(
        firefox_dir,
        String::from("waynr/firefox"),
        String::from("v0"),
        Vec::new(),
        vec![
            Box::new(Firefox{}),
            Box::new(aspects::PulseAudio{}),
            Box::new(aspects::X11{}),
            Box::new(aspects::Video{}),
            Box::new(aspects::DBus{}),
            Box::new(aspects::NetHost{}),
        ],
        vec![
            "/opt/firefox/firefox-bin",
            "--no-remote",
            "--profile", format!("{}/.mozilla/firefox/profile", home).as_str(),
        ].into_iter()
            .map(String::from)
            .collect(),
    );

    mgr.execute("firefox");
}
