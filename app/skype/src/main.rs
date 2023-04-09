use std::env;

use anyhow::{Context, Result};

use dfiles::aspects;
use dfiles::containermanager::ContainerManager;

#[derive(Clone)]
struct Skype {}

impl aspects::ContainerAspect for Skype {
    fn name(&self) -> String {
        String::from("Skype")
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
                content: r#"# Add the skype debian repo
RUN curl -sSL https://repo.skype.com/data/SKYPE-GPG-KEY | apt-key add -
RUN echo "deb [arch=amd64] https://repo.skype.com/deb stable main" > /etc/apt/sources.list.d/skype.list

RUN apt-get update && apt-get -y install \
    --no-install-recommends \
        skypeforlinux \
    && apt-get purge --autoremove \
    && rm -rf /var/lib/apt/lists/*
"#.to_string(),
            },
            aspects::DockerfileSnippet {
                order: 92,
                content: r#"COPY /run-skype-and-wait-for-exit /usr/local/bin
RUN chmod 755 /usr/local/bin/run-skype-and-wait-for-exit"#.to_string(),
            },
        ]
    }
    fn container_files(&self) -> Vec<aspects::ContainerFile> {
        vec![
            aspects::ContainerFile {
                container_path: String::from("./run-skype-and-wait-for-exit"),
                contents: r#"#!/bin/bash
skypeforlinux
sleep 3
while ps -C skypeforlinux >/dev/null;do sleep 3;done "#
                    .into(),
            },
            aspects::ContainerFile {
                container_path: String::from("./etc/fonts/local.conf"),
                contents: r#"<?xml version='1.0'?>
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
</fontconfig>"#
                    .into(),
            },
        ]
    }
}

fn main() -> Result<()> {
    let home = env::var("HOME").expect("HOME must be set");
    let container_path = format!("{}/.config/skypeforlinux", home);

    let version = env!("CARGO_PKG_VERSION");

    let mut mgr = ContainerManager::default_debian(
        "skype".to_string(),
        vec![format!("{}:{}", "waynr/skype", version)],
        vec![container_path],
        vec![
            Box::new(Skype {}),
            Box::new(aspects::Name("skype".to_string())),
            Box::new(aspects::CurrentUser::detect().context("detecting current user")?),
            Box::new(aspects::PulseAudio {}),
            Box::new(aspects::X11 {}),
            Box::new(aspects::Video {}),
            Box::new(aspects::DBus {}),
            Box::new(aspects::SysAdmin {}),
            Box::new(aspects::Shm {}),
        ],
        vec!["run-skype-and-wait-for-exit"]
            .into_iter()
            .map(String::from)
            .collect(),
        None,
    )?;

    mgr.execute().context("executing skype in container")
}
