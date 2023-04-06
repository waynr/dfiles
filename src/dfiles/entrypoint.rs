use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use super::aspects;
use super::error::Result;

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
    scripts.sort_by(|a, b| a.order.partial_cmp(&b.order).unwrap() );
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
    let scripts: Vec<ScriptSnippet> = aspects
        .iter()
        .map(|a| a.entrypoint_scripts())
        .flatten()
        .collect();

    if scripts.len() == 0 {
        return Ok(Vec::new());
    }

    write_scripts(&tmpdir, scripts)?;
    println!("{}", tmpdir.display());
    run_args(&tmpdir)
}
