use std::convert::TryFrom;
use std::fmt;
use std::path::Path;
use std::{env, fs};

use clap::{Arg, ArgMatches};
use serde::{Deserialize, Serialize};
use users;

pub struct DockerfileSnippet {
    pub order: u8,
    pub content: String,
}

pub struct ContainerFile {
    pub container_path: String,
    pub contents: String,
}

pub trait ContainerAspect {
    fn name(&self) -> String;
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String>;
    fn config_args(&self) -> Vec<Arg> {
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
        let pulsedir = format!("{}/{}", xdg_runtime_dir, "pulse");

        vec![
            "-v",
            format!("{0}/.pulse:{0}/.pulse", home).as_str(),
            "-v",
            format!("{0}/.config/pulse:{0}/.config/pulse", home).as_str(),
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
                    r#"COPY /etc/pulse/client.conf /etc/pulse/client.conf
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
            container_path: String::from("./etc/pulse/client.conf"),
            contents: String::from(
                "# Connect to the host's server using the mounted UNIX socket
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
        let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR must be set");

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

    fn config_args(&self) -> Vec<Arg> {
        vec![Arg::with_name("profile")
            .short("p")
            .long("profile")
            .help("specify the profile to use")
            .takes_value(true)
            .default_value("default")]
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mount {
    pub host_path: String,
    pub container_path: String,
}

impl ContainerAspect for Mount {
    fn name(&self) -> String {
        String::from("Mount")
    }
    fn run_args(&self, _matches: Option<&ArgMatches>) -> Vec<String> {
        vec![
            "-v",
            format!("{}:{}", self.host_path, self.container_path).as_str(),
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }
}

impl TryFrom<&str> for Mount {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let vs: Vec<&str> = value.split(':').collect();
        if vs.len() != 2 {
            return Err("invalid mount string");
        }
        Ok(Mount {
            host_path: vs[0].to_string(),
            container_path: vs[1].to_string(),
        })
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

    fn config_args(&self) -> Vec<Arg> {
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

pub struct CurrentUser {}

impl ContainerAspect for CurrentUser {
    fn name(&self) -> String {
        let uid = users::get_current_uid();
        let user = users::get_user_by_uid(uid).unwrap();
        format!("User: {}", user.name().to_str().unwrap())
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        Vec::new()
    }
    fn dockerfile_snippets(&self) -> Vec<DockerfileSnippet> {
        let uid = users::get_current_uid();
        let gid = users::get_current_gid();
        let user = users::get_user_by_uid(uid).unwrap();
        let group = users::get_group_by_gid(gid).unwrap();
        vec![
            DockerfileSnippet {
                order: 80,
                content: format!(
                    r#"RUN addgroup --gid {gid} {group} \
    &&  adduser --home /home/{user} \
                --shell /bin/bash \
                --uid {uid} \
                --gid {gid} \
                --disabled-password {user}
RUN adduser {user} audio
RUN adduser {user} video
RUN mkdir -p /data && chown {user}.{user} /data
RUN mkdir -p /home/{user} && chown {user}.{user} /home/{user}
"#,
                    gid = gid,
                    group = group.name().to_str().unwrap(),
                    user = user.name().to_str().unwrap(),
                    uid = uid,
                ),
            },
            DockerfileSnippet {
                order: 98,
                content: format!("USER {}", user.name().to_str().unwrap()),
            },
        ]
    }
}

// TODO: Locale should detect the host's locale settings and transfer those into the container at
// build time; should probably be configurable by command line flag but we don't yet support
// built-time command line flags and I'm feeling really lazy and just want to dispense entirely
// with my old base docker images so for now it's only configurable at compile time.
pub struct Locale {
    pub language: String,
    pub territory: String,
    pub codeset: String,
}

impl ContainerAspect for Locale {
    fn name(&self) -> String {
        format!("AutoLocale")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        Vec::new()
    }
    fn dockerfile_snippets(&self) -> Vec<DockerfileSnippet> {
        let locale = format!("{}_{}.{}", self.language, self.territory, self.codeset);
        vec![DockerfileSnippet {
            order: 88,
            content: format!(
                r#"RUN echo '{locale} {codeset}' > /etc/locale.gen
RUN locale-gen
RUN echo LANG="{locale}" > /etc/default/locale
ENV LANG={locale}"#,
                locale = locale,
                codeset = self.codeset,
            ),
        }]
    }
}

pub struct Timezone(pub String);

impl ContainerAspect for Timezone {
    fn name(&self) -> String {
        format!("Timezone")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        Vec::new()
    }
    fn dockerfile_snippets(&self) -> Vec<DockerfileSnippet> {
        vec![DockerfileSnippet {
            order: 88,
            content: format!(
                r#"ENV TZ={tz}
RUN ln -snf /usr/share/zoneinfo/{tz} /etc/localtime
RUN echo {tz} > /etc/timezone
"#,
                tz = self.0,
            ),
        }]
    }
}
