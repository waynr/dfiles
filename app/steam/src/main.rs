use std::env;

use clap::ArgMatches;

use dfiles::aspects;
use dfiles::containermanager::default_debian_container_manager;

struct Steam {}

impl aspects::ContainerAspect for Steam {
    fn name(&self) -> String {
        String::from("steam")
    }

    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        Vec::new()
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

fn main() {
    let home = env::var("HOME").expect("HOME must be set");
    let host_path_prefix = format!("{}/.steam/", home);
    let container_path = format!("{}/.steam/", home);

    let host_downloads_path = format!("{}/.local/share/Steam", home);
    let container_downloads_path = format!("{}/.local/share/Steam", home);

    let version = env!("CARGO_PKG_VERSION");

    let mut mgr = default_debian_container_manager(
        vec![format!("{}:{}", "waynr/steam", version)],
        vec![
            Box::new(Steam {}),
            Box::new(aspects::Name("steam".to_string())),
            Box::new(aspects::CurrentUser {}),
            Box::new(aspects::Locale {
                language: "en".to_string(),
                territory: "US".to_string(),
                codeset: "UTF-8".to_string(),
            }),
            Box::new(aspects::Timezone("America/Chicago".to_string())),
            Box::new(aspects::PulseAudio {}),
            Box::new(aspects::Alsa {}),
            Box::new(aspects::X11 {}),
            Box::new(aspects::Video {}),
            Box::new(aspects::DBus {}),
            Box::new(aspects::NetHost {}),
            Box::new(aspects::Shm {}),
            Box::new(aspects::Profile {
                host_path_prefix: host_path_prefix,
                container_path: container_path,
            }),
            Box::new(aspects::Mount {
                host_path: host_downloads_path,
                container_path: container_downloads_path,
            }),
        ],
        vec!["/usr/games/steam"]
            .into_iter()
            .map(String::from)
            .collect(),
    );

    mgr.execute("steam");
}
