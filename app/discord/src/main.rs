use std::env;

use anyhow::{Context, Result};

use dfiles::aspects;
use dfiles::containermanager::ContainerManager;

#[derive(Clone)]
struct Discord {}

impl aspects::ContainerAspect for Discord {
    fn name(&self) -> String {
        String::from("discord")
    }

    fn dockerfile_snippets(&self) -> Vec<aspects::DockerfileSnippet> {
        vec![
            aspects::DockerfileSnippet {
                order: 91,
                content: r#"WORKDIR /opt/
RUN curl https://dl.discordapp.net/apps/linux/0.0.34/discord-0.0.34.deb > /opt/discord.deb && \
    dpkg --force-depends -i /opt/discord.deb  ; rm /opt/discord.deb
RUN apt-get update && apt-get --fix-broken install -y \
  && apt-get purge --autoremove \
  && rm -rf /var/lib/apt/lists/* \
  && rm -rf /src/*.deb "#
                    .to_string(),
            },
            aspects::DockerfileSnippet {
                order: 92,
                content: String::from(
                    r#"RUN apt-get update && apt-get install -y \
    --no-install-recommends \
    libxshmfence1 \
    libgbm1 \
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
    let container_path = format!("{}/.config/discord/", home);

    let version = env!("CARGO_PKG_VERSION");

    let mut mgr = ContainerManager::default_debian(
        "discord".to_string(),
        vec![format!("{}:{}", "waynr/discord", version)],
        vec![container_path],
        vec![
            Box::new(Discord {}),
            Box::new(aspects::Name("discord".to_string())),
            Box::new(aspects::CurrentUser::detect().context("detecting current user")?),
            Box::new(aspects::PulseAudio {}),
            Box::new(aspects::X11 {}),
            Box::new(aspects::Video {}),
            Box::new(aspects::DBus {}),
            Box::new(aspects::SysAdmin {}),
            Box::new(aspects::Shm {}),
        ],
        vec!["discord"].into_iter().map(String::from).collect(),
        None,
    )?;

    mgr.execute().context("executing discord in container")
}
