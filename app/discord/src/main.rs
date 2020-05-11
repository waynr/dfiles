use std::env;

use clap::ArgMatches;

use dfiles::aspects;
use dfiles::containermanager::default_debian_container_manager;

struct Discord {}

impl aspects::ContainerAspect for Discord {
    fn name(&self) -> String {
        String::from("discord")
    }

    fn run_args(&self, _: Option<&ArgMatches>) -> Vec<String> {
        Vec::new()
    }

    fn dockerfile_snippets(&self) -> Vec<aspects::DockerfileSnippet> {
        vec![aspects::DockerfileSnippet {
            order: 91,
            content: format!(
                r#"WORKDIR /opt/
RUN curl https://dl.discordapp.net/apps/linux/0.0.10/discord-0.0.10.deb > /opt/discord.deb && \
    dpkg --force-depends -i /opt/discord.deb  ; rm /opt/discord.deb
RUN apt-get update && apt-get --fix-broken install -y \
  && apt-get purge --autoremove \
  && rm -rf /var/lib/apt/lists/* \
  && rm -rf /src/*.deb "#,
            ),
        }]
    }
}

fn main() {
    let home = env::var("HOME").expect("HOME must be set");
    let host_path_prefix = format!("{}/.config/discord/", home);
    let container_path = format!("{}/.config/discord/", home);

    let host_downloads_path = format!("{}/downloads", home);
    let container_downloads_path = format!("{}/Downloads", home);

    let host_visual_path = format!("{}/visual", home);
    let container_visual_path = format!("{}/visual", home);

    let version = env!("CARGO_PKG_VERSION");

    let mut mgr = default_debian_container_manager(
        "discord".to_string(),
        vec![format!("{}:{}", "waynr/discord", version)],
        vec![
            Box::new(Discord {}),
            Box::new(aspects::Name("discord".to_string())),
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
            Box::new(aspects::SysAdmin {}),
            Box::new(aspects::Shm {}),
            Box::new(aspects::CPUShares("512".to_string())),
            Box::new(aspects::Memory("3072mb".to_string())),
            Box::new(aspects::Profile {
                host_path_prefix: host_path_prefix,
                container_path: container_path,
            }),
            Box::new(aspects::Mount {
                host_path: host_visual_path,
                container_path: container_visual_path,
            }),
            Box::new(aspects::Mount {
                host_path: host_downloads_path,
                container_path: container_downloads_path,
            }),
        ],
        vec!["discord"].into_iter().map(String::from).collect(),
    );

    mgr.execute();
}
