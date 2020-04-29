use std::env;

use clap::ArgMatches;

use dfiles::aspects;
use dfiles::containermanager::default_debian_container_manager;

struct Firefox {}

impl aspects::ContainerAspect for Firefox {
    fn name(&self) -> String {
        String::from("firefox")
    }

    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        Vec::new()
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
                    release = "75.0"
                ),
            },
            aspects::DockerfileSnippet {
                order: 90,
                content: String::from(
                    r#"RUN apt-get update && apt-get install -y \
    --no-install-recommends \
    firefox-esr \
  && apt-get purge --autoremove \
  && rm -rf /var/lib/apt/lists/* \
  && rm -rf /src/*.deb "#,
                ),
            },
        ]
    }
}

fn main() {
    let home = env::var("HOME").expect("HOME must be set");
    let host_path_prefix = format!("{}/.mozilla/firefox", home);
    let container_path = format!("{}/.mozilla/firefox/profile", home);

    let host_downloads_path = format!("{}/downloads", home);
    let container_downloads_path = format!("{}/Downloads", home);

    let host_visual_path = format!("{}/visual", home);
    let container_visual_path = format!("{}/visual", home);

    let version = env!("CARGO_PKG_VERSION");

    let mut mgr = default_debian_container_manager(
        vec![format!("{}:{}", "waynr/firefox", version)],
        vec![
            Box::new(Firefox {}),
            Box::new(aspects::Name("firefox".to_string())),
            Box::new(aspects::CurrentUser {}),
            Box::new(aspects::Locale {
                language: "en".to_string(),
                territory: "US".to_string(),
                codeset: "UTF-8".to_string(),
            }),
            Box::new(aspects::Timezone("America/Chicago".to_string())),
            Box::new(aspects::PulseAudio {}),
            Box::new(aspects::X11 {}),
            Box::new(aspects::Video {}),
            Box::new(aspects::DBus {}),
            Box::new(aspects::NetHost {}),
            Box::new(aspects::Shm {}),
            Box::new(aspects::CPUShares("512".to_string())),
            Box::new(aspects::Memory("3072mb".to_string())),
            Box::new(aspects::Profile {
                host_path_prefix: host_path_prefix,
                container_path: container_path,
            }),
            Box::new(aspects::Mounts(vec![aspects::Mount(
                host_visual_path,
                container_visual_path,
            )])),
            Box::new(aspects::Mounts(vec![aspects::Mount(
                host_downloads_path,
                container_downloads_path,
            )])),
        ],
        vec![
            "/opt/firefox/firefox-bin",
            "--no-remote",
            "--profile",
            format!("{}/.mozilla/firefox/profile", home).as_str(),
        ]
        .into_iter()
        .map(String::from)
        .collect(),
    );

    mgr.execute("firefox");
}
