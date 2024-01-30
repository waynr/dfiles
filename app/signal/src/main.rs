use std::env;

use anyhow::{Context, Result};

use dfiles::aspects;
use dfiles::containermanager::ContainerManager;

#[derive(Clone)]
struct Signal {}

impl aspects::ContainerAspect for Signal {
    fn name(&self) -> String {
        String::from("signal")
    }

    fn dockerfile_snippets(&self) -> Vec<aspects::DockerfileSnippet> {
        vec![
            aspects::DockerfileSnippet {
                order: 90,
                content: String::from(
                    r#"RUN apt-get update && apt-get install -y --no-install-recommends \
        libgtk-3-0 \
        libpango1.0-0 \
        libcanberra-gtk* \
        hicolor-icon-theme \
        libgl1-mesa-dri \
        libgl1-mesa-glx \
        libv4l-0 \
        openjdk-11-jre \
        fonts-symbola \
        xdg-utils \
    && curl -sSL https://updates.signal.org/desktop/apt/keys.asc | apt-key add - \
    && echo "deb [arch=amd64] https://updates.signal.org/desktop/apt xenial main" | tee -a /etc/apt/sources.list.d/signal-xenial.list \
    && apt-get update && apt-get install -y --no-install-recommends \
        signal-desktop \
    && rm -rf /var/lib/apt/lists/* \
    && rm -rf /src/*.deb "#,
                ),
            },
            aspects::DockerfileSnippet {
                order: 91,
                content: String::from(r#"RUN chmod 4755 /opt/Signal/chrome-sandbox"#),
            },
        ]
    }
}

fn main() -> Result<()> {
    let home = env::var("HOME").expect("HOME must be set");
    let container_path = format!("{}/.config/Signal/", home);

    let mut mgr = ContainerManager::default_debian(
        "signal".to_string(),
        vec![String::from("waynr/signal:v0")],
        vec![container_path],
        vec![
            Box::new(Signal {}),
            Box::new(aspects::Name("signal".to_string())),
            Box::new(aspects::PulseAudio {}),
            Box::new(aspects::CurrentUser::detect().context("detecting current user")?),
            Box::new(aspects::X11 {}),
            Box::new(aspects::Video {}),
            Box::new(aspects::DBus {}),
            Box::new(aspects::SysAdmin {}),
        ],
        vec!["/opt/Signal/signal-desktop"]
            .into_iter()
            .map(String::from)
            .collect(),
        Some("bullseye".to_string()),
    )?;

    mgr.execute().context("executing signal in container")
}
