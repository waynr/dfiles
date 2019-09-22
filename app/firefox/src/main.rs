use std::collections::HashMap;
use std::env;

use clap::{Arg, ArgMatches};
use tar::{Builder, Header};
use tempfile::NamedTempFile;

use dfiles::aspects;
use dfiles::containermanager::new_container_manager;

struct Firefox {}
impl aspects::ContainerAspect for Firefox {
    fn name(&self) -> String {
        String::from("Firefox")
    }
    fn run_args(&self, matches: Option<&ArgMatches>) -> Vec<String> {
        let home = env::var("HOME").expect("HOME must be set");

        let mut profile = "default";
        if let Some(m) = matches {
            if let Some(c) = m.value_of("profile") {
                profile = c
            }
        }

        vec![
            "-v",
            format!(
                "{h}/.mozilla/firefox/{p}:{h}/.mozilla/firefox/profile",
                h = home,
                p = profile
            )
            .as_str(),
            "-v",
            format!("{}/downloads:/home/wayne/Downloads", home).as_str(),
            "--name",
            format!("firefox-{}", profile).as_str(),
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    fn cli_run_args(&self) -> Vec<Arg> {
        vec![Arg::with_name("profile")
            .short("p")
            .long("profile")
            .help("specify the firefox profile to use")
            .takes_value(true)
            .default_value("default")]
        .into_iter()
        .collect()
    }
}

fn main() {
    let home = env::var("HOME").expect("HOME must be set");
    let tar_file = NamedTempFile::new().unwrap();
    let mut a = Builder::new(&tar_file);

    let mut context: HashMap<&str, &[u8]> = HashMap::new();
    context.insert("Dockerfile", include_bytes!("Dockerfile"));
    context.insert("pulse-client.conf", include_bytes!("pulse-client.conf"));
    for (name, bs) in context {
        let mut header = Header::new_gnu();
        header.set_path(name).unwrap();
        header.set_size(bs.len() as u64);
        header.set_cksum();
        a.append(&header, bs).unwrap();
    }

    let mgr = new_container_manager(
        tar_file.path().to_path_buf(),
        vec![String::from("waynr/firefox:v0")],
        Vec::new(),
        vec![
            Box::new(Firefox {}),
            Box::new(aspects::PulseAudio {}),
            Box::new(aspects::X11 {}),
            Box::new(aspects::Video {}),
            Box::new(aspects::DBus {}),
            Box::new(aspects::NetHost {}),
            Box::new(aspects::Shm {}),
            Box::new(aspects::CPUShares("512".to_string())),
            Box::new(aspects::Memory("3072mb".to_string())),
        ],
        vec![
            "/opt/firefox/firefox-bin",
            "--no-remote",
            "--profile",
            format!("{}/.mozilla/firefox/profile", home).as_str(),
        ]
        .into_iter()
        .map(String::from)
        .collect(),
    );

    mgr.execute("firefox");
}
