use std::env;

use clap::ArgMatches;

use dfiles::aspects;
use dfiles::containermanager::default_debian_container_manager;

struct Skype {}

impl aspects::ContainerAspect for Skype {
    fn name(&self) -> String {
        String::from("Skype")
    }

    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        Vec::new()
    }

    fn dockerfile_snippets(&self) -> Vec<aspects::DockerfileSnippet> {
        vec![
            aspects::DockerfileSnippet {
                order: 75,
                content: String::from(
                    r#"COPY /etc/fonts/local.conf /etc/fonts/local.conf
RUN chmod 655 /etc/fonts
RUN chmod 644 /etc/fonts/local.conf"#,
                ),
            },
            aspects::DockerfileSnippet {
                order: 91,
                content: format!(
                    r#"# Add the skype debian repo
RUN curl -sSL https://repo.skype.com/data/SKYPE-GPG-KEY | apt-key add -
RUN echo "deb [arch=amd64] https://repo.skype.com/deb stable main" > /etc/apt/sources.list.d/skype.list

RUN apt-get update && apt-get -y install \
    --no-install-recommends \
        skypeforlinux \
    && apt-get purge --autoremove \
    && rm -rf /var/lib/apt/lists/*
"#,
                ),
            },
            aspects::DockerfileSnippet {
                order: 92,
                content: format!(
                    r#"COPY /run-skype-and-wait-for-exit /usr/local/bin
RUN chmod 755 /usr/local/bin/run-skype-and-wait-for-exit"#,
                ),
            },
        ]
    }
    fn container_files(&self) -> Vec<aspects::ContainerFile> {
        vec![
            aspects::ContainerFile {
                container_path: String::from("./run-skype-and-wait-for-exit"),
                contents: String::from(
                    r#"#!/bin/bash
skypeforlinux
sleep 3
while ps -C skypeforlinux >/dev/null;do sleep 3;done "#,
                ),
            },
            aspects::ContainerFile {
                container_path: String::from("./etc/fonts/local.conf"),
                contents: String::from(
                    r#"<?xml version='1.0'?>
<!DOCTYPE fontconfig SYSTEM 'fonts.dtd'>
<fontconfig>
<match target="font">
<edit mode="assign" name="rgba">
<const>rgb</const>
</edit>
</match>
<match target="font">
<edit mode="assign" name="hinting">
<bool>true</bool>
</edit>
</match>
<match target="font">
<edit mode="assign" name="hintstyle">
<const>hintslight</const>
</edit>
</match>
<match target="font">
<edit mode="assign" name="antialias">
<bool>true</bool>
</edit>
</match>
<match target="font">
<edit mode="assign" name="lcdfilter">
<const>lcddefault</const>
</edit>
</match>
<match target="font">
<edit name="embeddedbitmap" mode="assign">
<bool>false</bool>
</edit>
</match>
</fontconfig>"#,
                ),
            },
        ]
    }
}

fn main() {
    let home = env::var("HOME").expect("HOME must be set");
    let host_path_prefix = format!("{}/.config/skypeforlinux", home);
    let container_path = format!("{}/.config/skypeforlinux", home);

    let version = env!("CARGO_PKG_VERSION");

    let mut mgr = default_debian_container_manager(
        "skype".to_string(),
        vec![format!("{}:{}", "waynr/skype", version)],
        vec![
            Box::new(Skype {}),
            Box::new(aspects::Name("skype".to_string())),
            Box::new(aspects::CurrentUser {}),
            Box::new(aspects::Locale {
                language: "en".to_string(),
                territory: "US".to_string(),
                codeset: "UTF-8".to_string(),
            }),
            Box::new(aspects::PulseAudio {}),
            Box::new(aspects::X11 {}),
            Box::new(aspects::Video {}),
            Box::new(aspects::DBus {}),
            Box::new(aspects::Network {
                mode: "host".to_string(),
            }),
            Box::new(aspects::SysAdmin {}),
            Box::new(aspects::Shm {}),
            Box::new(aspects::Profile {
                host_path_prefix: host_path_prefix,
                container_path: container_path,
            }),
        ],
        vec!["run-skype-and-wait-for-exit"]
            .into_iter()
            .map(String::from)
            .collect(),
    );

    mgr.execute();
}
