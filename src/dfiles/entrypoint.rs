use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process;

use which::which;

use super::aspects;
use super::error::Result;

pub struct Script {
    pub description: String,
    pub snippet: String,
}

const ENTRYPOINT_SETUP_SCRIPT_DIR: &str = "entrypoint_scripts";
const ENTRYPOINT_SETUP_SCRIPT: &str = "/entrypoint_scripts/top.bash";

/// Generates command line arguments to be used in `docker run` calls on the host system.
fn run_args(tmpdir: &Path) -> Result<Vec<String>> {
    let mut args = Vec::new();
    let binary = std::env::current_exe()?;
    args.extend(vec![
        String::from("-v"),
        format!("{}:{}", binary.to_string_lossy(), "/entrypoint"),
        String::from("-v"),
        format!(
            "{}:{}",
            tmpdir.join(ENTRYPOINT_SETUP_SCRIPT_DIR).display(),
            PathBuf::from("/")
                .join(ENTRYPOINT_SETUP_SCRIPT_DIR)
                .display(),
        ),
        String::from("--entrypoint"),
        String::from("/entrypoint"),
    ]);

    Ok(args)
}

fn write_scripts(tmpdir: &Path, scripts: Vec<Script>) -> Result<PathBuf> {
    let path = tmpdir.join(ENTRYPOINT_SETUP_SCRIPT_DIR).join("top.bash");
    std::fs::create_dir_all(path.parent().unwrap())?;
    let mut file = std::fs::File::create(&path)?;
    file.metadata()?.permissions().set_mode(0o700);

    write!(file, "#!/usr/bin/env bash\n")?;
    write!(file, "\nUSER=root\n")?;
    for script in scripts {
        write!(file, "\n")?;
        for line in script.description.lines() {
            write!(file, "# {0}\n", line)?;
        }

        write!(file, "{0}\n", script.snippet)?;
    }
    write!(file, "\n# execute whatever command was specified\n")?;
    write!(file, "sudo --user $USER $@\n")?;

    Ok(path)
}

pub(crate) fn setup(
    tmpdir: &Path,
    aspects: &Vec<Box<dyn aspects::ContainerAspect>>,
) -> Result<Vec<String>> {
    let scripts: Vec<Script> = aspects
        .iter()
        .map(|a| a.entrypoint_scripts())
        .flatten()
        .collect();

    if scripts.len() == 0 {
        return Ok(Vec::new());
    }

    write_scripts(&tmpdir, scripts)?;
    run_args(&tmpdir)
}

pub fn execute(cmd: Vec<String>) -> Result<()> {
    println!("entrypoint running {:?}", &cmd);

    process::Command::new(which("bash")?)
        .arg("-x")
        .arg(ENTRYPOINT_SETUP_SCRIPT)
        .args(cmd)
        .status()?;

    Ok(())
}
