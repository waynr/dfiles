use std::collections::HashMap;

use dfiles::containermanager::noop_container_manager;
use dfiles::containermanager::ContainerManager;

pub fn dfiles_files_container_mgr() -> ContainerManager {
    let mut context: HashMap<String, String> = HashMap::new();
    context.insert(
        "Dockerfile".to_string(),
        include_str!("dfilesfiles.dockerfile").to_string(),
    );
    context.insert(
        "pulse-client.conf".to_string(),
        include_str!("pulse-client.conf").to_string(),
    );

    let version = env!("CARGO_PKG_VERSION");

    noop_container_manager(context, vec![format!("{}:{}", "dfilesfiles", version)])
}
