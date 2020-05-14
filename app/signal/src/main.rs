use std::env;

use clap::ArgMatches;

use dfiles::aspects;
use dfiles::containermanager::default_debian_container_manager;

struct Signal {}

impl aspects::ContainerAspect for Signal {
    fn name(&self) -> String {
        String::from("signal")
    }

    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        Vec::new()
    }

    fn dockerfile_snippets(&self) -> Vec<aspects::DockerfileSnippet> {
        vec![aspects::DockerfileSnippet {
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
        }]
    }
}

fn main() {
    let home = env::var("HOME").expect("HOME must be set");
    let host_path_prefix = format!("{}/.config/signal/", home);
    let container_path = format!("{}/.config/Signal/", home);

    let mut mgr = default_debian_container_manager(
        "signal".to_string(),
        vec![String::from("waynr/signal:v0")],
        vec![
            Box::new(Signal {}),
            Box::new(aspects::Name("signal".to_string())),
            Box::new(aspects::Locale {
                language: "en".to_string(),
                territory: "US".to_string(),
                codeset: "UTF-8".to_string(),
            }),
            Box::new(aspects::PulseAudio {}),
            Box::new(aspects::CurrentUser {}),
            Box::new(aspects::X11 {}),
            Box::new(aspects::Video {}),
            Box::new(aspects::DBus {}),
            Box::new(aspects::Network {
                mode: "host".to_string(),
            }),
            Box::new(aspects::SysAdmin {}),
            Box::new(aspects::Shm {}),
            Box::new(aspects::Profile {
                host_path_prefix: host_path_prefix,
                container_path: container_path,
            }),
        ],
        vec!["/opt/Signal/signal-desktop"]
            .into_iter()
            .map(String::from)
            .collect(),
    );

    mgr.execute();
}
