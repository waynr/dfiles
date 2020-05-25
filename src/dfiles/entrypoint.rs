use std::process;

use which::which;

use super::error::{Error, Result};

pub struct Script {
    pub description: String,
    pub sudo_args: Vec<String>,
    pub script: String,
}

pub struct Entrypoint {
    scripts: Vec<Script>,
}

impl Entrypoint {
    pub fn empty() -> Self {
        Self {
            scripts: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        return self.scripts.len() == 0;
    }

    pub fn load() -> Self {
        Entrypoint::empty()
    }

    pub fn prepare(&self) -> Result<()> {
        Ok(())
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

    pub fn execute(&self, args: Vec<String>) -> Result<()> {
        let sudo_path = which("sudo")?;
        let mut sudo_args = Vec::new();

        for script in &self.scripts {
            sudo_args.extend(&script.sudo_args);
        }

        if args.len() < 2 {
            return Err(Error::MissingEntrypointArgs);
        }

        println!("entrypoint: running {:?}", &args[1..]);
        process::Command::new(sudo_path)
            .args(sudo_args)
            .arg("--")
            .args(&args[1..])
            .status()?;
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
