use std::env;

use anyhow::{Context, Result};

use dfiles::aspects;
use dfiles::containermanager::ContainerManager;

#[derive(Clone)]
struct Thinkorswim {}

impl aspects::ContainerAspect for Thinkorswim {
    fn name(&self) -> String {
        String::from("thinkorswim")
    }

    fn dockerfile_snippets(&self) -> Vec<aspects::DockerfileSnippet> {
        vec![
            aspects::DockerfileSnippet {
                order: 90,
                content: format!(
                    r#"
RUN apt-get update && apt-get install -y --no-install-recommends \
        openjdk-11-jre \
    && rm -rf /var/lib/apt/lists/* \
    && rm -rf /src/*.deb"#,
                ),
            },
            aspects::DockerfileSnippet {
                order: 91,
                content: format!(
                    r#"WORKDIR /opt/
ADD https://mediaserver.thinkorswim.com/installer/InstFiles/thinkorswim_installer.sh ./
RUN bash -x thinkorswim_installer.sh -q
"#,
                ),
            },
        ]
    }
}

fn main() -> Result<()> {
    let home = env::var("HOME").expect("HOME must be set");

    let version = env!("CARGO_PKG_VERSION");

    let mut mgr = ContainerManager::default_debian(
        "thinkorswim".to_string(),
        vec![format!("{}:{}", "waynr/thinkorswim", version)],
        vec![
            format!("{}/.thinkorswim", home),
            format!("{}/.java", home),
            format!("{}/.install4j", home),
            format!("{}/.com.devexperts.tos.ui.user.login.ThinkOrSwimApplication", home),
        ],
        vec![
            Box::new(Thinkorswim {}),
            Box::new(aspects::Name("thinkorswim".to_string())),
            Box::new(aspects::CurrentUser::detect().context("detecting current user")?),
            Box::new(aspects::PulseAudio {}),
            Box::new(aspects::X11 {}),
            Box::new(aspects::Video {}),
            Box::new(aspects::DBus {}),
            Box::new(aspects::Shm {}),
        ],
        vec![
            "/opt/thinkorswim/thinkorswim",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        Some(String::from("bullseye")),
    );

    mgr.execute().context("executing thinkorswim in container")
}
