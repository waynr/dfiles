use std::env;

use anyhow::{Context, Result};

use dfiles::aspects;
use dfiles::containermanager::ContainerManager;

#[derive(Clone)]
struct Chrome {}

impl aspects::ContainerAspect for Chrome {
    fn name(&self) -> String {
        String::from("Chrome")
    }

    fn dockerfile_snippets(&self) -> Vec<aspects::DockerfileSnippet> {
        vec![
            aspects::DockerfileSnippet {
                order: 91,
                content: r#"
ADD https://dl.google.com/linux/direct/google-talkplugin_current_amd64.deb /src/google-talkplugin_current_amd64.deb
RUN apt-get update && apt-get install -y --no-install-recommends \
        libpango1.0-0 \
        libcanberra-gtk* \
        hicolor-icon-theme \
        libgl1-mesa-dri \
        libgl1-mesa-glx \
        libv4l-0 \
        openjdk-11-jre \
        fonts-symbola \
    && curl -sSL https://dl.google.com/linux/linux_signing_key.pub | apt-key add - \
    && echo "deb [arch=amd64] https://dl.google.com/linux/chrome/deb/ stable main" > /etc/apt/sources.list.d/google.list \
    && apt-get update && apt-get install -y --no-install-recommends \
        google-chrome-stable \
    && dpkg -i /src/google-talkplugin_current_amd64.deb \
    && apt-get purge --auto-remove -y curl \
    && rm -rf /var/lib/apt/lists/* \
    && rm -rf /src/*.deb"#.to_string(),
            },
            aspects::DockerfileSnippet {
                order: 75,
                content: String::from(
                    r#"COPY /etc/fonts/local.conf /etc/fonts/local.conf
RUN chmod 655 /etc/fonts
RUN chmod 644 /etc/fonts/local.conf"#,
                ),
            },
        ]
    }
    fn container_files(&self) -> Vec<aspects::ContainerFile> {
        vec![aspects::ContainerFile {
            container_path: String::from("./etc/fonts/local.conf"),
            contents: Vec::from(
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
        }]
    }
}

fn main() -> Result<()> {
    let data_dir = String::from("/data");
    let home = env::var("HOME").expect("HOME must be set");
    let home_dir = format!("{}/.config", home);
    let version = env!("CARGO_PKG_VERSION");

    let mut mgr = ContainerManager::default_debian(
        "chrome".to_string(),
        vec![format!("waynr/chrome:{}", version)],
        vec![home_dir, data_dir.clone()],
        vec![
            Box::new(Chrome {}),
            Box::new(aspects::Name("chrome".to_string())),
            Box::new(aspects::CurrentUser::detect().context("detecting current user")?),
            Box::new(aspects::PulseAudio {}),
            Box::new(aspects::X11 {}),
            Box::new(aspects::Video {}),
            Box::new(aspects::DBus {}),
            Box::new(aspects::SysAdmin {}),
            Box::new(aspects::Shm {}),
        ],
        vec!["google-chrome", &format!("--user-data-dir={}", data_dir)]
            .into_iter()
            .map(String::from)
            .collect(),
        Some(String::from("bullseye")),
    )?;

    mgr.execute().context("executing chrome in container")
}
