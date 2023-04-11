use std::env;

use anyhow::{Context, Result};

use dfiles_core::aspects;
use dfiles_core::containermanager::ContainerManager;

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

impl Signal {
    pub fn container_manager() -> Result<ContainerManager> {
        let home = env::var("HOME").expect("HOME must be set");
        let container_path = format!("{}/.config/Signal/", home);

        ContainerManager::default_debian(
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
            None,
        )
        .context("initializing signal container manager")
    }
}

fn main() -> Result<()> {
    let mut mgr = Signal::container_manager()?;
    let cli = &mut mgr
        .cli()
        .context(format!("initializing {0} cli Command", mgr.name()))?;
    mgr.execute(cli)
        .context(format!("executing {0} in container", mgr.name()))
}
