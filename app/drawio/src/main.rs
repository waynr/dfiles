use std::env;

use anyhow::{Context, Result};

use dfiles::aspects;
use dfiles::containermanager::ContainerManager;

#[derive(Clone)]
struct Drawio {}

impl aspects::ContainerAspect for Drawio {
    fn name(&self) -> String {
        String::from("drawio")
    }

    fn dockerfile_snippets(&self) -> Vec<aspects::DockerfileSnippet> {
        vec![aspects::DockerfileSnippet {
            order: 91,
            content: r#"WORKDIR /opt/
RUN curl -L https://github.com/jgraph/drawio-desktop/releases/download/v16.5.1/drawio-amd64-16.5.1.deb > /opt/drawio.deb && \
    dpkg --force-depends -i /opt/drawio.deb  ; rm /opt/drawio.deb
RUN apt-get update && apt-get --fix-broken install -y \
  && apt-get purge --autoremove \
  && rm -rf /var/lib/apt/lists/* \
  && rm -rf /src/*.deb "#.into(),
        },
            aspects::DockerfileSnippet {
                order: 92,
                content: String::from(
                    r#"RUN apt-get update && apt-get install -y \
    --no-install-recommends \
    libgbm1 \
    libasound2 \
  && apt-get purge --autoremove \
  && rm -rf /var/lib/apt/lists/* \
  && rm -rf /src/*.deb "#,
                ),
            },
        ]
    }
}

fn main() -> Result<()> {
    let home = env::var("HOME").expect("HOME must be set");
    let container_path = format!("{}/.config/draw.io/", home);

    let version = env!("CARGO_PKG_VERSION");

    let mut mgr = ContainerManager::default_debian(
        "drawio".to_string(),
        vec![format!("{}:{}", "waynr/drawio", version)],
        vec![container_path],
        vec![
            Box::new(Drawio {}),
            Box::new(aspects::Name("drawio".to_string())),
            Box::new(aspects::CurrentUser::detect().context("detecting current user")?),
            Box::new(aspects::PulseAudio {}),
            Box::new(aspects::X11 {}),
            Box::new(aspects::SysAdmin {}),
            //Box::new(aspects::Shm {}),
        ],
        vec!["drawio"].into_iter().map(String::from).collect(),
        None,
    );

    mgr.execute().context("executing drawio in container")
}
