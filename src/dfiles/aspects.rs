use clap::{Arg, ArgMatches};
use std::fmt;
use std::path::Path;
use std::{env, fs};

pub struct DockerfileSnippet {
    pub order: u8,
    pub content: String,
}

pub struct ContainerFile {
    container_path: String,
    contents: String,
}

pub trait ContainerAspect {
    fn name(&self) -> String;
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String>;
    fn cli_run_args(&self) -> Vec<Arg> {
        Vec::new()
    }
    fn cli_build_args(&self) -> Vec<Arg> {
        Vec::new()
    }
    fn dockerfile_snippets(&self) -> Vec<DockerfileSnippet> {
        Vec::new()
    }
    fn container_files(&self) -> Vec<ContainerFile> {
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
    fn name(&self) -> String {
        String::from("PulseAudio")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        let home = env::var("HOME").expect("HOME must be set");
        let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").expect("HOME must be set");

        let mut pulsedir = String::new();
        for entry in fs::read_dir(xdg_runtime_dir.clone()).unwrap() {
            let entry = entry.unwrap();
            let filename: String = entry.file_name().into_string().unwrap();
            let pathstring: String = entry.path().into_os_string().into_string().unwrap();
            match filename.as_str() {
                x if x.starts_with("pulse") => pulsedir = pathstring,
                _ => continue,
            }
        }

        vec![
            "-v",
            format!("{0}/.pulse:{0}/.pulse", home).as_str(),
            "-v",
            format!("{0}:{0}", pulsedir).as_str(),
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }
    fn dockerfile_snippets(&self) -> Vec<DockerfileSnippet> {
        vec![
            DockerfileSnippet {
                order: 75,
                content: String::from(
                    r#"COPY /pulse-client.conf /etc/pulse/client.conf
RUN chmod 655 /etc/pulse
RUN chmod 644 /etc/pulse/client.conf"#,
                ),
            },
            DockerfileSnippet {
                order: 70,
                content: String::from(
                    r#"RUN apt-get update && apt-get install -y \
    --no-install-recommends \
    libpulse0 \
  && apt-get purge --autoremove \
  && rm -rf /var/lib/apt/lists/* \
  && rm -rf /src/*.deb "#,
                ),
            },
        ]
    }
    fn container_files(&self) -> Vec<ContainerFile> {
        vec![ContainerFile {
            container_path: String::from("/etc/pulse/client.conf"),
            contents: String::from(
                "
# Connect to the host's server using the mounted UNIX socket
default-server = unix:/run/user/11571/pulse/native

# Prevent a server running in the container
autospawn = no
daemon-binary = /bin/true

# Prevent the use of shared memory
enable-shm = false
            ",
            ),
        }]
    }
}

pub struct Alsa {}
impl ContainerAspect for Alsa {
    fn name(&self) -> String {
        String::from("Alsa")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        vec!["--device", "/dev/snd"]
            .into_iter()
            .map(String::from)
            .collect()
    }
}

pub struct X11 {}
impl ContainerAspect for X11 {
    fn name(&self) -> String {
        String::from("X11")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        let display = env::var("DISPLAY").expect("DISPLAY must be set");

        vec![
            "-e",
            format!("DISPLAY=unix{}", display).as_str(),
            "-v",
            "/tmp/.X11-unix:/tmp/.X11-unix",
            "--device",
            "/dev/dri",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }
}

pub struct Video {}
impl ContainerAspect for Video {
    fn name(&self) -> String {
        String::from("Video")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        let video_devices: Vec<String> = fs::read_dir(Path::new("/dev"))
            .expect("get entries for dir")
            .filter_map(Result::ok)
            .filter(|entry| match entry.path().file_name() {
                Some(x) => match x.to_os_string().into_string() {
                    Ok(x) => x.starts_with(&"video"),
                    Err(_) => false,
                },
                None => false,
            })
            .map(|e| e.path().as_os_str().to_os_string().into_string())
            .filter_map(Result::ok)
            .collect();

        video_devices
            .iter()
            .map(|s| vec![String::from("--device"), s.to_string()])
            .flatten()
            .collect()
    }
}

pub struct DBus {}
impl ContainerAspect for DBus {
    fn name(&self) -> String {
        String::from("DBus")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        let home = env::var("HOME").expect("HOME must be set");
        let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").expect("HOME must be set");

        vec![
            "-v",
            format!("{0}/bus:{0}/bus", xdg_runtime_dir).as_str(),
            "-v",
            format!("{0}:{0}", "/var/run/dbus/system_bus_socket").as_str(),
            "-v",
            format!("{0}/.dbus/session-bus:{0}/.dbus/session-bus", home).as_str(),
            "-e",
            format!("DBUS_SESSION_BUS_ADDRESS=unix:path={}/bus", xdg_runtime_dir).as_str(),
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }
    fn dockerfile_snippets(&self) -> Vec<DockerfileSnippet> {
        vec![DockerfileSnippet {
            order: 71,
            content: String::from(
                r#"RUN apt-get update && apt-get install -y \
    --no-install-recommends \
    dbus-x11 \
  && apt-get purge --autoremove \
  && rm -rf /var/lib/apt/lists/* \
  && rm -rf /src/*.deb "#,
            ),
        }]
    }
}

pub struct NetHost {}
impl ContainerAspect for NetHost {
    fn name(&self) -> String {
        String::from("NetHost")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        vec!["--net", "host"]
            .into_iter()
            .map(String::from)
            .collect()
    }
}

pub struct SysAdmin {}
impl ContainerAspect for SysAdmin {
    fn name(&self) -> String {
        String::from("SysAdmin")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        vec!["--cap-add", "SYS_ADMIN"]
            .into_iter()
            .map(String::from)
            .collect()
    }
}

pub struct TTY {}
impl ContainerAspect for TTY {
    fn name(&self) -> String {
        String::from("TTY")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        vec!["-i", "-t"].into_iter().map(String::from).collect()
    }
}

pub struct Shm {}
impl ContainerAspect for Shm {
    fn name(&self) -> String {
        String::from("Shm")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        vec!["-v", "/dev/shm:/dev/shm"]
            .into_iter()
            .map(String::from)
            .collect()
    }
}

pub struct CPUShares(pub String);
impl ContainerAspect for CPUShares {
    fn name(&self) -> String {
        String::from("CPUShares")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        vec!["--cpu-shares", self.0.as_str()]
            .into_iter()
            .map(String::from)
            .collect()
    }
}

pub struct Memory(pub String);
impl ContainerAspect for Memory {
    fn name(&self) -> String {
        String::from("Memory")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        vec!["--memory", self.0.as_str()]
            .into_iter()
            .map(String::from)
            .collect()
    }
}

pub struct Profile {
    pub host_path_prefix: String,
    pub container_path: String,
}
impl ContainerAspect for Profile {
    fn name(&self) -> String {
        String::from("Profile")
    }
    fn run_args(&self, matches: Option<&ArgMatches>) -> Vec<String> {
        let mut profile = "default";
        if let Some(m) = matches {
            if let Some(c) = m.value_of("profile") {
                profile = c
            }
        }

        let host_path = format!("{}/{}", self.host_path_prefix, profile);

        vec![
            "-v",
            format!("{}:{}", host_path, self.container_path).as_str(),
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    fn cli_run_args(&self) -> Vec<Arg> {
        vec![Arg::with_name("profile")
            .short("p")
            .long("profile")
            .help("specify the profile to use")
            .takes_value(true)
            .default_value("default")]
        .into_iter()
        .collect()
    }
}

pub struct Mount(pub String, pub String);

pub struct Mounts(pub Vec<Mount>);
impl ContainerAspect for Mounts {
    fn name(&self) -> String {
        String::from("Mounts")
    }
    fn run_args(&self, _matches: Option<&ArgMatches>) -> Vec<String> {
        let mounts = &self.0;
        mounts
            .into_iter()
            .map(|m| vec![String::from("--volume"), format!("{}:{}", m.0, m.1)])
            .flatten()
            .collect()
    }
}

pub struct Name(pub String);
impl ContainerAspect for Name {
    fn name(&self) -> String {
        String::from("Name")
    }
    fn run_args(&self, matches: Option<&ArgMatches>) -> Vec<String> {
        let mut container_name: String = "default".to_string();
        if let Some(m) = matches {
            if let Some(c) = m.value_of("container_name") {
                container_name = c.to_string();
            } else if let Some(c) = m.value_of("profile") {
                container_name = format!("{}-{}", self.0, c);
            }
        }
        vec!["--name".to_string(), container_name]
            .into_iter()
            .collect()
    }

    fn cli_run_args(&self) -> Vec<Arg> {
        vec![Arg::with_name("container_name")
            .short("n")
            .long("name")
            .help("specify the name of the container to be run")
            .global(true)
            .takes_value(true)]
        .into_iter()
        .collect()
    }
}
