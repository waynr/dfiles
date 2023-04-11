use std::env;

use anyhow::{Context, Result};

use dfiles::aspects;
use dfiles::containermanager::ContainerManager;

#[derive(Clone)]
struct Zoom {}

impl aspects::ContainerAspect for Zoom {
    fn name(&self) -> String {
        String::from("zoom")
    }

    fn dockerfile_snippets(&self) -> Vec<aspects::DockerfileSnippet> {
        vec![aspects::DockerfileSnippet {
            order: 91,
            content: r#"WORKDIR /opt/
RUN curl -L https://zoom.us/client/latest/zoom_amd64.deb -o /opt/zoom_amd64.deb && \
    dpkg --force-depends -i /opt/zoom_amd64.deb && rm /opt/zoom_amd64.deb
RUN apt-get update && apt-get --fix-broken install -y \
  && apt-get purge --autoremove \
  && rm -rf /var/lib/apt/lists/* \
  && rm -rf /src/*.deb "#
                .to_string(),
        }]
    }
}

impl Zoom {
    fn container_manager() -> Result<ContainerManager> {
        let home = env::var("HOME").expect("HOME must be set");

        let version = env!("CARGO_PKG_VERSION");

        Ok(ContainerManager::default_debian(
            "zoom".to_string(),
            vec![format!("{}:{}", "waynr/zoom", version)],
            vec![
                format!("{}/.zoom/", home),
                format!("{}/.config/zoomus.conf", home),
            ],
            vec![
                Box::new(Zoom {}),
                Box::new(aspects::Name("zoom".to_string())),
                Box::new(aspects::CurrentUser::detect().context("detecting current user")?),
                Box::new(aspects::PulseAudio {}),
                Box::new(aspects::X11 {}),
                Box::new(aspects::Video {}),
                Box::new(aspects::DBus {}),
                Box::new(aspects::SysAdmin {}),
                Box::new(aspects::Shm {}),
            ],
            vec!["zoom"].into_iter().map(String::from).collect(),
            None,
        )
        .context("initializing zoom container manager")?)
    }
}

fn main() -> Result<()> {
    let mut mgr = Zoom::container_manager()?;
    let cli = &mut mgr
        .cli()
        .context(format!("initializing {0} cli Command", mgr.name()))?;
    mgr.execute(cli)
        .context(format!("executing {0} in container", mgr.name()))
}
