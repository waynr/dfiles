use std::fmt;
use std::path::{
    Path,
};
use std::{
    env,
    fs,
};
use clap::{
    Arg,
    ArgMatches,
};

pub trait ContainerAspect {
    fn name(&self) -> String;
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String>;
    fn cli_run_args(&self) -> Vec<Arg> {
        Vec::new()
    }
    fn cli_build_args(&self) -> Vec<Arg> {
        Vec::new()
    }
}

impl fmt::Display for dyn ContainerAspect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {:?}", self.name(), self.run_args(None))
    }
}

pub struct PulseAudio {}
impl ContainerAspect for PulseAudio {
    fn name(&self) -> String { String::from("PulseAudio") }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        let home = env::var("HOME")
            .expect("HOME must be set");
        let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR")
            .expect("HOME must be set");

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

        vec![
            "-v", format!("{0}/.pulse:{0}/.pulse", home).as_str(),
            "-v", format!("{0}:{0}", pulsedir).as_str(),
        ].into_iter()
            .map(String::from)
            .collect()
    }
}

pub struct X11 {}
impl ContainerAspect for X11 {
    fn name(&self) -> String { String::from("X11") }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        let display = env::var("DISPLAY")
            .expect("DISPLAY must be set");

        vec![
            "-e", format!("DISPLAY=unix{}", display).as_str(),
            "-v", "/tmp/.X11-unix:/tmp/.X11-unix",
            "--device", "/dev/dri",
        ].into_iter()
            .map(String::from)
            .collect()
    }
}

pub struct Video {}
impl ContainerAspect for Video {
    fn name(&self) -> String { String::from("Video") }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        let video_devices: Vec<String> = fs::read_dir(Path::new("/dev"))
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

        video_devices.iter()
            .map(|s| vec![String::from("--device"), s.to_string()])
            .flatten()
            .collect()
    }
}

pub struct DBus {}
impl ContainerAspect for DBus {
    fn name(&self) -> String { String::from("DBus") }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        let home = env::var("HOME")
            .expect("HOME must be set");
        let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR")
            .expect("HOME must be set");

        vec![
            "-v", format!("{0}/bus:{0}/bus", xdg_runtime_dir).as_str(),
            "-v", format!("{0}:{0}", "/var/run/dbus/system_bus_socket").as_str(),
            "-v", format!("{0}/.dbus/session-bus:{0}/.dbus/session-bus", home).as_str(),
            "-e", format!("DBUS_SESSION_BUS_ADDRESS=unix:path={}/bus", xdg_runtime_dir).as_str(),
        ].into_iter()
            .map(String::from)
            .collect()
    }
}

pub struct NetHost {}
impl ContainerAspect for NetHost {
    fn name(&self) -> String { String::from("NetHost") }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        vec![
            "--net", "host",
        ].into_iter()
            .map(String::from)
            .collect()
    }
}

pub struct SysAdmin {}
impl ContainerAspect for SysAdmin {
    fn name(&self) -> String { String::from("SysAdmin") }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        vec![
            "--cap-add", "SYS_ADMIN",
        ].into_iter()
            .map(String::from)
            .collect()
    }
}

pub struct TTY {}
impl ContainerAspect for TTY {
    fn name(&self) -> String { String::from("TTY") }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        vec![
            "-i", "-t",
        ].into_iter()
            .map(String::from)
            .collect()
    }
}

pub struct Shm {}
impl ContainerAspect for Shm {
    fn name(&self) -> String { String::from("Shm") }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        vec![
            "-v", "/dev/shm:/dev/shm",
        ].into_iter()
            .map(String::from)
            .collect()
    }
}

pub struct CPUShares(pub String);
impl ContainerAspect for CPUShares {
    fn name(&self) -> String { String::from("CPUShares") }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        vec![
            "--cpu-shares", self.0.as_str(),
        ].into_iter()
            .map(String::from)
            .collect()
    }
}

pub struct Memory(pub String);
impl ContainerAspect for Memory {
    fn name(&self) -> String { String::from("Memory") }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        vec![
            "--memory", self.0.as_str(),
        ].into_iter()
            .map(String::from)
            .collect()
    }
}
