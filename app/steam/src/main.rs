use std::env;

use anyhow::{Context, Result};

use dfiles::aspects;
use dfiles::containermanager::ContainerManager;

struct Steam {}

impl aspects::ContainerAspect for Steam {
    fn name(&self) -> String {
        String::from("steam")
    }

    fn dockerfile_snippets(&self) -> Vec<aspects::DockerfileSnippet> {
        vec![aspects::DockerfileSnippet {
            order: 91,
            content: format!(
                r#"RUN dpkg --add-architecture i386
RUN sed -i -e 's|main|main contrib non-free|' /etc/apt/sources.list
RUN apt-get update && yes 'I AGREE' | apt-get install -y \
        steam \
    && apt-get purge --autoremove \
    && rm -rf /var/lib/apt/lists/* \
    && rm -rf /src/*.deb "#,
            ),
        }]
    }
}

fn main() -> Result<()> {
    let home = env::var("HOME").expect("HOME must be set");
    let container_path = format!("{}/.steam/", home);

    let version = env!("CARGO_PKG_VERSION");

    let mut mgr = ContainerManager::default_debian(
        "stream".to_string(),
        vec![format!("{}:{}", "waynr/steam", version)],
        vec![container_path],
        vec![
            Box::new(Steam {}),
            Box::new(aspects::Name("steam".to_string())),
            Box::new(aspects::CurrentUser::detect().context("detecting current user")?),
            Box::new(aspects::Locale {
                language: "en".to_string(),
                territory: "US".to_string(),
                codeset: "UTF-8".to_string(),
            }),
            Box::new(aspects::PulseAudio {}),
            Box::new(aspects::Alsa {}),
            Box::new(aspects::X11 {}),
            Box::new(aspects::Video {}),
            Box::new(aspects::DBus {}),
            Box::new(aspects::Shm {}),
        ],
        vec!["/usr/games/steam"]
            .into_iter()
            .map(String::from)
            .collect(),
    );

    mgr.execute()
}
