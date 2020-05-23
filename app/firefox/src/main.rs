use std::env;

use anyhow::{Context, Result};

use dfiles::aspects;
use dfiles::containermanager::ContainerManager;

#[derive(Clone)]
struct Firefox {}

const VERSION: &'static str = "109.0.1";

impl aspects::ContainerAspect for Firefox {
    fn name(&self) -> String {
        String::from("firefox")
    }

    fn dockerfile_snippets(&self) -> Vec<aspects::DockerfileSnippet> {
        vec![
            aspects::DockerfileSnippet {
                order: 91,
                content: format!(
                    r#"WORKDIR /opt/
ADD https://archive.mozilla.org/pub/firefox/releases/{release}/linux-x86_64/en-US/firefox-{release}.tar.bz2 ./
RUN tar -xjvf /opt/firefox-{release}.tar.bz2
RUN ln -sf /opt/firefox/firefox-bin /usr/local/bin/firefox"#,
                    release = VERSION,
                ),
            },
            aspects::DockerfileSnippet {
                order: 90,
                content: String::from(
                    r#"RUN apt-get update && apt-get install -y \
    --no-install-recommends \
    firefox-esr \
    libasound2 \
    libxt6 \
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
    let container_path = format!("{}/.mozilla/firefox/profile", home);

    let mut mgr = ContainerManager::default_debian(
        "firefox".to_string(),
        vec![format!("{}:{}", "waynr/firefox", VERSION)],
        vec![container_path.clone()],
        vec![
            Box::new(Firefox {}),
            Box::new(aspects::Name("firefox".to_string())),
            Box::new(
                aspects::CurrentUser::detect(aspects::CurrentUserMode::Builtin)
                    .context("detecting current user")?,
            ),
            Box::new(aspects::PulseAudio {}),
            Box::new(aspects::X11 {}),
            Box::new(aspects::Video {}),
            Box::new(aspects::DBus {}),
            Box::new(aspects::Shm {}),
        ],
        vec![
            "/opt/firefox/firefox-bin",
            "--no-remote",
            "--profile",
            &container_path,
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        Some(String::from("bookworm")),
    );

    mgr.execute().context("executing firefox in container")
}
