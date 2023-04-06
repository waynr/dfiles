use std::env;

use anyhow::{Context, Result};

use dfiles::aspects;
use dfiles::containermanager::ContainerManager;

#[derive(Clone)]
struct Thinkorswim {
    user_name: String,
}

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
RUN chown -R {name}.{name} /opt/thinkorswim_installer.sh
RUN chmod u+wrx /opt/thinkorswim_installer.sh
"#,
                    name = self.user_name,
                ),
            },
            aspects::DockerfileSnippet {
                order: 97,
                content: format!(
                    r#"
COPY ./opt/run-thinkorswim.sh /run-thinkorswim.sh
RUN chown -R {name}.{name} /run-thinkorswim.sh
RUN chmod u+wrx /run-thinkorswim.sh
ENTRYPOINT ["/run-thinkorswim.sh"]
"#,
                    name = self.user_name,
                ),
            },
        ]
    }

    fn container_files(&self) -> Vec<aspects::ContainerFile> {
        vec![aspects::ContainerFile {
            container_path: String::from("./opt/run-thinkorswim.sh"),
            contents: std::include_str!("entrypoint.sh").into(),
        }]
    }
}

fn main() -> Result<()> {
    let home = env::var("HOME").expect("HOME must be set");

    let version = env!("CARGO_PKG_VERSION");

    let current_user = aspects::CurrentUser::detect().context("detecting current user")?;
    let tos = Thinkorswim {
        user_name: current_user.name(),
    };

    let thinkorswim_install_dir = format!("{}/thinkorswim", home);
    let mut mgr = ContainerManager::default_debian(
        "thinkorswim".to_string(),
        vec![format!("{}:{}", "waynr/thinkorswim", version)],
        vec![
            format!("{}/.thinkorswim", home),
            format!("{}/.java", home),
            format!("{}/.install4j", home),
            thinkorswim_install_dir.clone(),
            format!(
                "{}/.com.devexperts.tos.ui.user.login.ThinkOrSwimApplication",
                home
            ),
        ],
        vec![
            Box::new(tos),
            Box::new(aspects::Name("thinkorswim".to_string())),
            Box::new(current_user),
            Box::new(aspects::PulseAudio {}),
            Box::new(aspects::X11 {}),
            Box::new(aspects::Video {}),
            Box::new(aspects::DBus {}),
            Box::new(aspects::Shm {}),
        ],
        vec![format!("{}/thinkorswim", thinkorswim_install_dir)],
        Some(String::from("bullseye")),
    )?;

    mgr.execute().context("executing thinkorswim in container")
}
