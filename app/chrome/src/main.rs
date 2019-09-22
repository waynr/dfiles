use clap::ArgMatches;
use std::env;
use std::path::PathBuf;

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
    let chrome_dir = PathBuf::from("/home/wayne/projects/dockerfiles/chrome");

    let mgr = new_container_manager(
        chrome_dir,
        String::from("waynr/chrome"),
        String::from("v0"),
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
