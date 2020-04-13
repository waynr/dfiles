use std::collections::HashMap;
use std::env;

use dfiles::aspects;
use dfiles::containermanager::new_container_manager;
use dfilesfiles::dfiles_files_container_mgr;

fn main() {
    let mut context: HashMap<String, String> = HashMap::new();
    context.insert(
        "Dockerfile".to_string(),
        include_str!("signal.dockerfile").to_string(),
    );

    let home = env::var("HOME").expect("HOME must be set");
    let host_path_prefix = format!("{}/.config/signal/", home);
    let container_path = format!("{}/.config/Signal/", home);

    let host_downloads_path = format!("{}/downloads", home);
    let container_downloads_path = format!("{}/Downloads", home);

    let dfilesfiles_mgr = dfiles_files_container_mgr();

    let mut mgr = new_container_manager(
        context,
        vec![String::from("waynr/signal:v0")],
        vec![Box::new(dfilesfiles_mgr)],
        vec![
            Box::new(aspects::Name("signal".to_string())),
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
            Box::new(aspects::Mounts(vec![aspects::Mount(
                host_downloads_path,
                container_downloads_path,
            )])),
        ],
        vec!["/opt/Signal/signal-desktop"]
            .into_iter()
            .map(String::from)
            .collect(),
    );

    mgr.execute("signal");
}
