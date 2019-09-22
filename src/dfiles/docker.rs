use std::process::Command;

use serde::Deserialize;
use serde_json::from_value;
use shiplift::BuildOptions;
use shiplift::Docker;
use tokio::prelude::Future;
use tokio::prelude::Stream;

#[derive(Deserialize, Debug)]
struct BuildOutput {
    stream: String,
}

pub fn build(opts: &BuildOptions) {
    let docker = Docker::new();

    let fut = docker
        .images()
        .build(opts)
        .for_each(|output| {
            let u: Result<BuildOutput, _> = from_value(output);
            match u {
                Ok(a) => print!("{}", a.stream),
                Err(_) => (),
            }
            Ok(())
        })
        .map_err(|e| eprintln!("Error: {}", e));

    tokio::run(fut);
}

pub fn run(args: Vec<String>) {
    let cmdstr: String = args.join(" ");
    println!("docker {}", cmdstr);

    let mut child = Command::new("docker")
        .arg("run")
        .args(args)
        .spawn()
        .expect("meow");

    let _ = child.wait().expect("failed waiting for child process");
}
