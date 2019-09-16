use std::path::{
    PathBuf,
};
use std::{
    env,
};
use clap::{
    Arg,
    ArgMatches,
};

use dfiles::aspects;
use dfiles::containermanager::{
    new_container_manager,
};

struct Firefox {}
impl aspects::ContainerAspect for Firefox {
    fn name(&self) -> String { String::from("Firefox") }
    fn run_args(&self, matches: Option<&ArgMatches>) -> Vec<String> {
        let home = env::var("HOME")
            .expect("HOME must be set");

        let mut profile = "default";
        if let Some(m) = matches {
            if let Some(c) = m.value_of("profile") {
                profile = c
            }
        }

        vec![
            "--cpu-shares", "512",
            "--memory", "3072mb",
            "-v", "/dev/shm:/dev/shm",

            "-v", format!("{h}/.mozilla/firefox/{p}:{h}/.mozilla/firefox/profile", h=home, p=profile).as_str(),
            "-v", format!("{}/downloads:/home/wayne/Downloads", home).as_str(),

            "--name", format!("firefox-{}", profile).as_str(),
        ].into_iter()
            .map(String::from)
            .collect()
    }

    fn cli_run_args(&self) -> Vec<Arg> {
        vec![
            Arg::with_name("profile")
                .short("p")
                .long("profile")
                .help("specify the firefox profile to use")
                .takes_value(true)
                .default_value("default"),
        ].into_iter()
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
