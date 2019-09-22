use std::collections::HashMap;
use std::env;

use clap::ArgMatches;
use tar::{Builder, Header};
use tempfile::NamedTempFile;

use dfiles::aspects;
use dfiles::containermanager::new_container_manager;

struct Chrome {}
impl aspects::ContainerAspect for Chrome {
    fn name(&self) -> String {
        String::from("Chrome")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        let home = env::var("HOME").expect("HOME must be set");

        vec![
            "-v",
            format!("{}/.config/google-chrome:/data", home).as_str(),
            "-v",
            format!("{}/downloads:/home/wayne/Downloads", home).as_str(),
            "--name",
            "chrome",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }
}

fn main() {
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
        vec![String::from("waynr/chrome:v0")],
        Vec::new(),
        vec![
            Box::new(Chrome {}),
            Box::new(aspects::PulseAudio {}),
            Box::new(aspects::X11 {}),
            Box::new(aspects::Video {}),
            Box::new(aspects::DBus {}),
            Box::new(aspects::NetHost {}),
            Box::new(aspects::SysAdmin {}),
            Box::new(aspects::Shm {}),
            Box::new(aspects::CPUShares("512".to_string())),
            Box::new(aspects::Memory("3072mb".to_string())),
        ],
        vec!["google-chrome", "--user-data-dir=/data"]
            .into_iter()
            .map(String::from)
            .collect(),
    );

    mgr.execute("chrome");
}
