use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use super::aspects;
use super::error::{Error, Result};

pub struct ScriptSnippet {
    pub description: String,
    pub order: u16,
    pub snippet: String,
}

const ENTRYPOINT_SETUP_SCRIPT: &str = "entrypoint.bash";

/// Generates command line arguments to be used in `docker run` calls on the host system.
fn run_args(tmpdir: &Path) -> Result<Vec<String>> {
    let mut args = Vec::new();
    let local_entrypoint_script = tmpdir.join(ENTRYPOINT_SETUP_SCRIPT);
    let container_entrypoint_script = PathBuf::from("/").join(ENTRYPOINT_SETUP_SCRIPT);
    args.extend(vec![
        String::from("-v"),
        format!(
            "{}:{}",
            local_entrypoint_script.display(),
            container_entrypoint_script.display()
        ),
        String::from("--entrypoint"),
        String::from("/entrypoint.bash"),
    ]);

    Ok(args)
}

fn write_scripts(tmpdir: &Path, mut scripts: Vec<ScriptSnippet>) -> Result<PathBuf> {
    let path = tmpdir.join(ENTRYPOINT_SETUP_SCRIPT);
    std::fs::create_dir_all(path.parent().unwrap())?;
    let mut file = std::fs::File::create(&path)?;

    write!(file, "#!/usr/bin/env bash\n")?;
    write!(file, "\nUSER=root\n")?;
    scripts.sort_by(|a, b| a.order.partial_cmp(&b.order).unwrap());
    for script in scripts {
        write!(file, "\n")?;
        for line in script.description.lines() {
            write!(file, "# {0}\n", line)?;
        }

        write!(file, "{0}\n", script.snippet)?;
    }
    write!(file, "\n# execute whatever command was specified\n")?;
    write!(file, "sudo --user $USER $@\n")?;

    let mut perms = file.metadata()?.permissions();
    perms.set_mode(0o700);
    file.set_permissions(perms)?;

    Ok(path)
}

pub(crate) fn setup(
    tmpdir: &Path,
    aspects: &Vec<Box<dyn aspects::ContainerAspect>>,
) -> Result<Vec<String>> {
    let mut result = Ok(Vec::new());
    let scripts: Vec<ScriptSnippet> = aspects
        .iter()
        .map_while(|a| match a.entrypoint_snippets() {
            Ok(v) => Some(v),
            Err(e) => {
                result = Err(e);
                None
            }
        })
        .flatten()
        .collect();

    if let Err(_) = result {
        return result;
    }

    if scripts.len() == 0 {
        return result;
    }

    write_scripts(&tmpdir, scripts)?;
    log::trace!("entrypoint tmpdir: {}", tmpdir.display());
    run_args(&tmpdir)
}

pub fn group_setup(group_name: &str) -> Result<ScriptSnippet> {
    let name = match users::get_current_username() {
        Some(n) => n.to_string_lossy().to_string(),
        None => return Err(Error::MissingUser("<unknown>".to_string())),
    };
    let video_group = match users::get_group_by_name(group_name) {
        Some(id) => id,
        None => return Err(Error::MissingGroup(group_name.to_string())),
    };
    Ok(ScriptSnippet {
        order: 5,
        description: "configure video group for container user".to_string(),
        snippet: String::from(format!(
            r#"
adduser {user} {group_name}
groupmod -g {video_gid} {group_name}
        "#,
            user = name,
            group_name = video_group.name().to_string_lossy(),
            video_gid = video_group.gid(),
        )),
    })
}
