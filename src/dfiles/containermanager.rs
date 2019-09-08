use std::path::{
    PathBuf,
    Path,
};
use std::{
    env,
    fs,
};


use clap::{
    App,
    Arg,
    SubCommand,
};

use shiplift::BuildOptions;

use super::docker;

pub struct PulseAudio {}
pub struct X11 {}
pub struct Video {}
pub struct DBus {}

pub enum ContainerAspects {
    PulseAudio,
    X11,
    Video,
    DBus,
}

pub struct ContainerManager {
    path: PathBuf,
    image_name: String,
    image_tag: String,
    dependencies: Vec<Box<ContainerManager>>,
    aspects: Vec<ContainerAspects>,
}

pub fn NewContainerManager(
    path: PathBuf,
    image_name: String,
    image_tag: String,
    dependencies: Vec<Box<ContainerManager>>,
    aspects: Vec<ContainerAspects>,
) -> ContainerManager {
    ContainerManager{
        path: path,
        image_name: image_name,
        image_tag: image_tag,
        aspects: aspects,
        dependencies: dependencies,
    }
}

impl ContainerManager {
    fn image(&self) -> String {
        String::from(format!("{}:{}", self.image_name, &self.image_tag))
    }

    fn run(&self) -> Result<(), ()> {
        let home = env::var("HOME")
            .expect("HOME must be set");
        let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR")
            .expect("HOME must be set");

        let display = env::var("DISPLAY")
            .expect("DISPLAY must be set");

        let mut pulsedir = String::new();

        for entry in fs::read_dir(xdg_runtime_dir.clone()).unwrap() {
            let entry = entry.unwrap();
            let filename: String = entry
                .file_name()
                .into_string()
                .unwrap();
            let pathstring: String = entry.path()
                .into_os_string()
                .into_string()
                .unwrap();
            match filename.as_str() {
                x if x.starts_with("pulse") => pulsedir = pathstring,
                _ => continue,
            }
        }


        let mut video_devices: Vec<String> = fs::read_dir(Path::new("/dev"))
            .expect("get entries for dir")
            .filter_map(Result::ok)
            .filter(|entry| match entry
                .path()
                .file_name() {
                    Some(x) => match x
                        .to_os_string()
                        .into_string() {
                            Ok(x) => x.starts_with(&"video"),
                            Err(_) => false,
                        },
                    None => false,
            })
            .map(|e| e.path()
                .as_os_str()
                .to_os_string()
                .into_string()
                )
            .filter_map(Result::ok)
            .collect();

        let mut args: Vec<String> = vec![
            "-i", "-t", "--rm",
            "--net", "host",

            "--cpu-shares", "512",
            "--memory", "3072mb",
            "--cap-add", "SYS_ADMIN",
            "-v", "/dev/shm:/dev/shm",

            "-v", format!("{}/.config/google-chrome:/data", home).as_str(),
            "-v", format!("{}/downloads:/home/wayne/Downloads", home).as_str(),


            "-e", format!("DISPLAY=unix{}", display).as_str(),
            "-v", "/tmp/.X11-unix:/tmp/.X11-unix",
            "--device", "/dev/dri",

            "-v", format!("{home}/.pulse:{home}/.pulse", home = home).as_str(),
            "-v", format!("{0}:{0}", pulsedir).as_str(),

            "-v", format!("{0}/bus:{0}/bus", xdg_runtime_dir).as_str(),
            "-v", format!("{0}:{0}", "/var/run/dbus/system_bus_socket").as_str(),
            "-v", format!("{0}/.dbus/session-bus:{0}/.dbus/session-bus", home).as_str(),
            "-e", format!("DBUS_SESSION_BUS_ADDRESS=unix:path={}/bus", xdg_runtime_dir).as_str(),

            //"-v", "/var/run:/var/run",
            //"-v", "/usr/bin/dbus-launch:/usr/bin/dbus-launch",
            "--name", "chrome",
        ].into_iter()
            .map(String::from)
            .collect();

        video_devices = video_devices
            .iter()
            .map(|s| vec![String::from("--device"), s.to_string()])
            .flatten()
            .collect();

        args = vec![args, video_devices].concat();
        args.push(self.image().to_string());
        args.push(String::from("google-chrome"));
        args.push(String::from("--user-data-dir=/data"));
        docker::run(args);
        Ok(())
    }

    fn build_deps(&self) {
        for dep in &self.dependencies {
            dep.build().unwrap();
        }
    }

    fn build(&self) -> Result<(), ()> {
        self.build_deps();
        let opts = &BuildOptions::builder(self.path.to_str().unwrap())
            .tag(self.image())
            .build();
        docker::build(opts);
        Ok(())
    }

    pub fn execute(&self, name: String) {
        let matches = App::new(name)
            .version("0.0")
            .subcommand(SubCommand::with_name("run"))
            .subcommand(SubCommand::with_name("build"))
            .get_matches();

        match matches.subcommand() {
            ("run", _) => self.run().unwrap(),
            ("build", _) => self.build().unwrap(),
            (_, _) => {
                self.build().unwrap();
                self.run().unwrap();
            }
        }
    }
}
