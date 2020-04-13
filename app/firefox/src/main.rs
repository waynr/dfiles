use std::collections::HashMap;
use std::env;

use dfiles::aspects;
use dfiles::containermanager::new_container_manager;
use dfilesfiles::dfiles_files_container_mgr;

fn main() {
    let mut context: HashMap<String, String> = HashMap::new();
    context.insert(
        "Dockerfile".to_string(),
        include_str!("firefox.dockerfile").to_string(),
    );

    let home = env::var("HOME").expect("HOME must be set");
    let host_path_prefix = format!("{}/.mozilla/firefox", home);
    let container_path = format!("{}/.mozilla/firefox/profile", home);

    let host_downloads_path = format!("{}/downloads", home);
    let container_downloads_path = format!("{}/Downloads", home);

    let host_visual_path = format!("{}/visual", home);
    let container_visual_path = format!("{}/visual", home);

    let version = env!("CARGO_PKG_VERSION");

    let dfilesfiles_mgr = dfiles_files_container_mgr();

    let mut mgr = new_container_manager(
        context,
        vec![format!("{}:{}", "waynr/firefox", version)],
        vec![Box::new(dfilesfiles_mgr)],
        vec![
            Box::new(aspects::Name("firefox".to_string())),
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
