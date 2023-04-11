use std::convert::TryFrom;
use std::fmt;
use std::path::Path;
use std::str::FromStr;
use std::{env, fs};

use clap::{Arg, ArgAction, ArgMatches};
use dyn_clone;
use serde::{Deserialize, Serialize};
use users;

use super::dirs;
use super::entrypoint;
use super::error::{Error, Result};

pub struct DockerfileSnippet {
    pub order: u8,
    pub content: String,
}

pub struct ContainerFile {
    pub container_path: String,
    pub contents: Vec<u8>,
}

pub trait ContainerAspect: dyn_clone::DynClone {
    fn name(&self) -> String;
    fn run_args(&self, _: Option<&ArgMatches>) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
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
    fn entrypoint_snippets(&self) -> Result<Vec<entrypoint::ScriptSnippet>> {
        Ok(Vec::new())
    }
}

dyn_clone::clone_trait_object!(ContainerAspect);

impl fmt::Display for dyn ContainerAspect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} - {:?}",
            self.name(),
            self.run_args(None).unwrap_or(Vec::new())
        )
    }
}

#[derive(Clone)]
pub struct PulseAudio {}
impl ContainerAspect for PulseAudio {
    fn name(&self) -> String {
        String::from("PulseAudio")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Result<Vec<String>> {
        let home = env::var("HOME").expect("HOME must be set");
        let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").expect("HOME must be set");
        let pulsedir = format!("{}/{}", xdg_runtime_dir, "pulse");

        Ok(vec![
            "-v",
            format!("{0}/.pulse:{0}/.pulse", home).as_str(),
            "-v",
            format!("{0}/.config/pulse:{0}/.config/pulse", home).as_str(),
            "-v",
            format!("{0}:{0}", pulsedir).as_str(),
        ]
        .into_iter()
        .map(String::from)
        .collect())
    }
    fn dockerfile_snippets(&self) -> Vec<DockerfileSnippet> {
        vec![DockerfileSnippet {
            order: 70,
            content: String::from(
                r#"RUN apt-get update && apt-get install -y \
    --no-install-recommends \
    libavcodec-extra \
    libpulse0 \
  && apt-get purge --autoremove \
  && rm -rf /var/lib/apt/lists/* \
  && rm -rf /src/*.deb "#,
            ),
        }]
    }
    fn entrypoint_snippets(&self) -> Result<Vec<entrypoint::ScriptSnippet>> {
        let uid = users::get_current_uid();
        Ok(vec![
            entrypoint::ScriptSnippet {
                description: "configure pulseaudio client to connect to host daemon".to_string(),
                order: 50,
                snippet: format!(
                    r#"cat << EOF > /etc/pulse/client.conf
# Connect to the host's server using the mounted UNIX socket
default-server = unix:/run/user/{uid}/pulse/native

# Prevent a server running in the container
autospawn = no
daemon-binary = /bin/true

# Prevent the use of shared memory
enable-shm = false
EOF

chmod 655 /etc/pulse
chmod 644 /etc/pulse/client.conf"#,
                    uid = uid,
                ),
            },
            entrypoint::group_setup("audio")?,
        ])
    }
}

#[derive(Clone)]
pub struct Alsa {}
impl ContainerAspect for Alsa {
    fn name(&self) -> String {
        String::from("Alsa")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Result<Vec<String>> {
        Ok(vec!["--device", "/dev/snd"]
            .into_iter()
            .map(String::from)
            .collect())
    }
    fn entrypoint_snippets(&self) -> Result<Vec<entrypoint::ScriptSnippet>> {
        Ok(vec![entrypoint::group_setup("audio")?])
    }
}

#[derive(Clone)]
pub struct X11 {}
impl ContainerAspect for X11 {
    fn name(&self) -> String {
        String::from("X11")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Result<Vec<String>> {
        let display = env::var("DISPLAY").expect("DISPLAY must be set");

        Ok(vec![
            "-e",
            format!("DISPLAY=unix{}", display).as_str(),
            "-v",
            "/tmp/.X11-unix:/tmp/.X11-unix",
            "--device",
            "/dev/dri",
        ]
        .into_iter()
        .map(String::from)
        .collect())
    }
    fn dockerfile_snippets(&self) -> Vec<DockerfileSnippet> {
        vec![DockerfileSnippet {
            order: 72,
            content: String::from(
                r#"RUN apt-get update && apt-get install -y \
    --no-install-recommends \
    libxtst6 \
  && apt-get purge --autoremove \
  && rm -rf /var/lib/apt/lists/* \
  && rm -rf /src/*.deb "#,
            ),
        }]
    }
}

#[derive(Clone)]
pub struct Video {}
impl ContainerAspect for Video {
    fn name(&self) -> String {
        String::from("Video")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Result<Vec<String>> {
        let video_devices: Vec<String> = fs::read_dir(Path::new("/dev"))
            .expect("get entries for dir")
            .filter_map(std::result::Result::ok)
            .filter(|entry| match entry.path().file_name() {
                Some(x) => match x.to_os_string().into_string() {
                    Ok(x) => x.starts_with("video"),
                    Err(_) => false,
                },
                None => false,
            })
            .map(|e| e.path().as_os_str().to_os_string().into_string())
            .filter_map(std::result::Result::ok)
            .collect();

        Ok(video_devices
            .iter()
            .flat_map(|s| vec![String::from("--device"), s.to_string()])
            .collect())
    }
    fn dockerfile_snippets(&self) -> Vec<DockerfileSnippet> {
        vec![DockerfileSnippet {
            order: 72,
            content: String::from(
                r#"RUN apt-get update && apt-get install -y \
    --no-install-recommends \
    libpci3 \
    libpciaccess0 \
    libegl1 \
    libgl1 \
  && apt-get purge --autoremove \
  && rm -rf /var/lib/apt/lists/* \
  && rm -rf /src/*.deb "#,
            ),
        }]
    }
    fn entrypoint_snippets(&self) -> Result<Vec<entrypoint::ScriptSnippet>> {
        Ok(vec![entrypoint::group_setup("video")?])
    }
}

#[derive(Clone)]
pub struct DBus {}
impl ContainerAspect for DBus {
    fn name(&self) -> String {
        String::from("DBus")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Result<Vec<String>> {
        let home = env::var("HOME").expect("HOME must be set");
        let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR must be set");

        Ok(vec![
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
        .collect())
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Network {
    pub mode: String,
}

impl ContainerAspect for Network {
    fn name(&self) -> String {
        String::from("Network")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Result<Vec<String>> {
        Ok(vec!["--net", &self.mode]
            .into_iter()
            .map(String::from)
            .collect())
    }
}

impl TryFrom<&String> for Network {
    type Error = Error;
    fn try_from(value: &String) -> Result<Self> {
        Ok(Network {
            mode: value.to_string(),
        })
    }
}

#[derive(Clone)]
pub struct SysAdmin {}
impl ContainerAspect for SysAdmin {
    fn name(&self) -> String {
        String::from("SysAdmin")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Result<Vec<String>> {
        Ok(vec!["--cap-add", "SYS_ADMIN"]
            .into_iter()
            .map(String::from)
            .collect())
    }
}

#[derive(Clone)]
pub struct Shm {}
impl ContainerAspect for Shm {
    fn name(&self) -> String {
        String::from("Shm")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Result<Vec<String>> {
        Ok(vec!["-v", "/dev/shm:/dev/shm"]
            .into_iter()
            .map(String::from)
            .collect())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CPUShares(pub String);
impl ContainerAspect for CPUShares {
    fn name(&self) -> String {
        String::from("CPUShares")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Result<Vec<String>> {
        Ok(vec!["--cpu-shares", self.0.as_str()]
            .into_iter()
            .map(String::from)
            .collect())
    }
}

impl TryFrom<&String> for CPUShares {
    type Error = Error;
    fn try_from(value: &String) -> Result<Self> {
        Ok(CPUShares(value.to_string()))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Memory(pub String);
impl ContainerAspect for Memory {
    fn name(&self) -> String {
        String::from("Memory")
    }
    fn run_args(&self, _: Option<&ArgMatches>) -> Result<Vec<String>> {
        Ok(vec!["--memory", self.0.as_str()]
            .into_iter()
            .map(String::from)
            .collect())
    }
}

impl TryFrom<&String> for Memory {
    type Error = Error;
    fn try_from(value: &String) -> Result<Self> {
        Ok(Memory(value.to_string()))
    }
}

#[derive(Clone)]
pub struct Profile {
    pub name: String,
    pub container_paths: Vec<String>,
}
impl ContainerAspect for Profile {
    fn name(&self) -> String {
        String::from("Profile")
    }

    fn run_args(&self, matches: Option<&ArgMatches>) -> Result<Vec<String>> {
        let mut profile = &String::from("default");
        if let Some(m) = matches {
            if let Some(c) = m.get_one::<String>("profile") {
                profile = c;
            }
        }

        let host_path = dirs::get_data_dir(Some(&self.name), Some(profile))?;

        let mut output: Vec<String> = Vec::new();
        for s in &self.container_paths {
            let mut s_path = Path::new(&s);
            if let Ok(v) = s_path.strip_prefix("/") {
                s_path = v
            }
            let p = host_path.join(s_path);
            fs::create_dir_all(&p)?;

            output.push("-v".to_string());
            output.push(format!("{}:{}", p.to_path_buf().to_string_lossy(), s))
        }

        Ok(output)
    }

    fn config_args(&self) -> Vec<Arg> {
        vec![Arg::new("profile")
            .short('p')
            .long("profile")
            .help("specify the profile to use")
            .action(ArgAction::Set)
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
    fn run_args(&self, _matches: Option<&ArgMatches>) -> Result<Vec<String>> {
        Ok(vec![
            "-v",
            format!("{}:{}", self.host_path, self.container_path).as_str(),
        ]
        .into_iter()
        .map(String::from)
        .collect())
    }
}

impl TryFrom<&String> for Mount {
    type Error = Error;
    fn try_from(value: &String) -> Result<Self> {
        let vs: Vec<&str> = value.split(':').collect();
        if vs.len() != 2 {
            return Err(Error::InvalidMount(value.to_string()));
        }
        Ok(Mount {
            host_path: vs[0].to_string(),
            container_path: vs[1].to_string(),
        })
    }
}

#[derive(Clone)]
pub struct Name(pub String);
impl ContainerAspect for Name {
    fn name(&self) -> String {
        String::from("Name")
    }
    fn run_args(&self, matches: Option<&ArgMatches>) -> Result<Vec<String>> {
        let mut container_name = String::from("default");
        if let Some(m) = matches {
            if let Some(c) = m.get_one::<String>("container_name") {
                container_name = c.to_string();
            } else if let Some(c) = m.get_one::<String>("profile") {
                container_name = format!("{}-{}", self.0, c);
            }
        }
        Ok(vec!["--name".to_string(), container_name]
            .into_iter()
            .collect())
    }

    fn config_args(&self) -> Vec<Arg> {
        vec![Arg::new("container_name")
            .short('n')
            .long("name")
            .help("specify the name of the container to be run")
            .global(true)
            .action(ArgAction::Set)]
        .into_iter()
        .collect()
    }
}

#[derive(Clone)]
pub struct CurrentUser {
    name: String,
}

impl CurrentUser {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn detect() -> Result<Self> {
        let uid = users::get_current_uid();
        let name = match users::get_user_by_uid(uid) {
            Some(n) => n.name().to_string_lossy().to_string(),
            None => return Err(Error::MissingUser(uid.to_string())),
        };
        Ok(Self { name })
    }
}

impl ContainerAspect for CurrentUser {
    fn name(&self) -> String {
        format!("User: {}", &self.name)
    }

    fn entrypoint_snippets(&self) -> Result<Vec<entrypoint::ScriptSnippet>> {
        let uid = users::get_current_uid();
        let gid = users::get_current_gid();
        let name = match users::get_user_by_uid(uid) {
            Some(n) => n.name().to_string_lossy().to_string(),
            None => return Err(Error::MissingUser(uid.to_string())),
        };
        let group = match users::get_user_by_uid(gid) {
            Some(g) => g.name().to_string_lossy().to_string(),
            None => return Err(Error::MissingGroup(gid.to_string())),
        };
        Ok(vec![entrypoint::ScriptSnippet {
            description: format!("create a user named {}", self.name),
            order: 2,
            snippet: format!(
                r#"addgroup --gid {gid} {group}
useradd --home-dir /home/{user} \
    --shell /bin/bash \
    --uid {uid} \
    --gid {gid} \
    {user}

# NOTE: I don't remember why this was necessary...
adduser {user} tty

mkdir -p /data /home/{user}
chown {user}:{group} /data /home/{user}

cd /home/{user}
USER={user}"#,
                gid = gid,
                group = group,
                user = name,
                uid = uid,
            ),
        }])
    }
}

/// Locale defaults to the buildtime specified locale (obtained from config). At runtime it will be
/// overridden by one of the following (in order of preference):
/// * LC_ALL env var
/// * LC_CTYPE env var
/// * LANG env var
/// * dfiles profile config
/// * dfiles app config
/// * dfiles global config
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Locale {
    pub language: String,
    pub territory: String,
    pub codeset: String,
}

impl ContainerAspect for Locale {
    fn name(&self) -> String {
        "AutoLocale".to_string()
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
    fn entrypoint_snippets(&self) -> Result<Vec<entrypoint::ScriptSnippet>> {
        let mut snippets = Vec::new();
        let mut locale = String::from(self);
        if let Some(value) = env::var("LC_ALL")
            .ok()
            .or_else(|| env::var("LC_CTYPE").ok())
            .or_else(|| env::var("LANG").ok())
        {
            locale = value;
        }

        snippets.push(entrypoint::ScriptSnippet {
            description: "set a non-default entrypoint snippet".to_string(),
            order: 80,
            snippet: format!(
                r#"echo '{locale} {codeset}' > /etc/locale.gen
locale-gen
echo LANG="{locale}" > /etc/default/locale
export LANG={locale}"#,
                locale = locale,
                codeset = self.codeset,
            ),
        });
        Ok(snippets)
    }
}

impl From<&Locale> for String {
    fn from(l: &Locale) -> String {
        format!("{0}_{1}.{2}", l.language, l.territory, l.codeset)
    }
}

impl TryFrom<&String> for Locale {
    type Error = Error;
    fn try_from(value: &String) -> Result<Self> {
        let mut locale = Locale {
            language: String::new(),
            territory: String::new(),
            codeset: String::new(),
        };
        let remainder: String;

        if let Some(i) = value.find('_') {
            let (left, right) = value.split_at(i);
            locale.language = left.to_string();
            let (_, right) = right.split_at(1);
            remainder = right.to_string();
        } else {
            return Err(Error::InvalidLocale(value.to_string()));
        }

        if let Some(i) = remainder.find('.') {
            let (left, right) = remainder.split_at(i);
            locale.territory = left.to_string();
            let (_, right) = right.split_at(1);
            locale.codeset = right.to_string();
        } else {
            return Err(Error::InvalidLocale(value.to_string()));
        }
        Ok(locale)
    }
}

#[cfg(test)]
mod locale_should {
    use super::*;

    #[test]
    fn convert_from_str() -> Result<()> {
        assert_eq!(
            Locale::try_from(&String::from("en_US.UTF-8"))?,
            Locale {
                language: "en".to_string(),
                territory: "US".to_string(),
                codeset: "UTF-8".to_string(),
            }
        );
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Timezone(pub String);

impl ContainerAspect for Timezone {
    fn name(&self) -> String {
        "Timezone".to_string()
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
    fn run_args(&self, matches: Option<&ArgMatches>) -> Result<Vec<String>> {
        let mut timezone = self.0.clone();
        if let Some(m) = matches {
            if let Some(tz) = m.get_one::<String>("timezone") {
                timezone = tz.to_string()
            }
        }

        let args: Vec<String> = vec!["-e".to_string(), format!("TZ={0}", timezone)];
        Ok(args)
    }
    fn entrypoint_snippets(&self) -> Result<Vec<entrypoint::ScriptSnippet>> {
        let tz = env::var("LC_ALL").ok().unwrap_or(self.0.clone());
        Ok(vec![
            entrypoint::ScriptSnippet {
                description: "configure timezone based on TZ variable in host".to_string(),
                order: 60,
                snippet: format!(
                    r#"export TZ={tz}
ln -snf /usr/share/zoneinfo/{tz} /etc/localtime
echo {tz} > /etc/timezone
"#,
                    tz = tz,
                ),
            },
            entrypoint::group_setup("audio")?,
        ])
    }
}

impl TryFrom<&String> for Timezone {
    type Error = Error;
    fn try_from(input: &String) -> Result<Self> {
        let tz = input.to_string();
        match chrono_tz::Tz::from_str(input) {
            Ok(_) => Ok(Timezone(tz)),
            Err(_) => Err(Error::InvalidTimezone(tz)),
        }
    }
}
