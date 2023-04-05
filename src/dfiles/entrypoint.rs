use std::cell::RefCell;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process;
use std::rc::Rc;

use which::which;

use super::error::{Error, Result};

pub struct Script {
    pub description: String,
    pub as_user: Option<String>,
    pub script: String,
}

pub struct Entrypoint {
    scripts: Vec<Script>,
    index: Rc<RefCell<u16>>,
}

impl Entrypoint {
    pub fn empty() -> Self {
        Self {
            scripts: Vec::new(),
            index: Rc::new(RefCell::new(0)),
        }
    }

    pub fn is_empty(&self) -> bool {
        return self.scripts.len() == 0;
    }

    pub fn load() -> Self {
        Entrypoint::empty()
    }

    pub fn run_args(&self) -> Result<Vec<String>> {
        let binary = std::env::current_exe()?;
        Ok(vec![
            String::from("-v"),
            format!("{}:{}", binary.to_string_lossy(), "/entrypoint"),
            String::from("--entrypoint"),
            String::from("/entrypoint"),
        ])
    }

    fn write_script(&self, s: &Script) -> Result<PathBuf> {
        let path = PathBuf::from(format!("/entrypoint_scripts/{}.bash", &self.index.borrow()));
        std::fs::create_dir_all(path.parent().unwrap())?;
        let mut file = std::fs::File::create(&path)?;
        file.metadata()?.permissions().set_mode(0o700);
        file.write(s.script.as_bytes())?;

        let mut i = self.index.borrow_mut();
        *i = *i + 1;

        Ok(path)
    }

    fn write_scripts(&self) -> Result<PathBuf> {
        let path = PathBuf::from("/entrypoint_scripts/top.bash");
        std::fs::create_dir_all(path.parent().unwrap())?;
        let mut file = std::fs::File::create(&path)?;
        file.metadata()?.permissions().set_mode(0o700);

        write!(file, "#!/usr/bin/env bash\n")?;
        for script in &self.scripts {
            let script_path = self.write_script(script)?;

            write!(file, "\n")?;
            for line in script.description.lines() {
                write!(file, "# {0}\n", line)?;
            }

            if let Some(username) = &script.as_user {
                write!(file, "sudo --user {0} ", username)?;
            }

            write!(file, "{0}\n", script_path.display())?;
        }
        write!(file, "\n# execute whatever command was specified\n")?;
        write!(file, "$@\n")?;

        Ok(path)
    }

    pub fn execute(&self, cmd: Vec<String>) -> Result<()> {
        println!("entrypoint running {:?}", &cmd);
        if self.scripts.len() > 0 {
            self.write_scripts()?;
            process::Command::new(which("bash")?)
                .arg("-x")
                .arg("/entrypoint_scripts/top.bash")
                .args(cmd)
                .status()?;
        } else {
            let (cmd, args) = cmd
                .split_first()
                .ok_or(Error::MustSpecifyContainerCommand)?;
            process::Command::new(cmd).args(args).status()?;
        }

        Ok(())
    }
}

impl Extend<Script> for Entrypoint {
    fn extend<T: IntoIterator<Item = Script>>(&mut self, iter: T) {
        for elem in iter {
            self.scripts.push(elem);
        }
    }
}
