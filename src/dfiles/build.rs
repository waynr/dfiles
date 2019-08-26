use shiplift::{BuildOptions, Docker};
use tokio::prelude::{Future, Stream};
use serde::Deserialize;
use serde_json::from_value;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
struct BuildOutput {
    stream: String,
}

pub fn build(path: PathBuf) {
    let docker = Docker::new();
    let path_str = path.to_str().unwrap();

    let fut = docker
        .images()
        .build(&BuildOptions::builder(path_str).tag("shiplift_test").build())
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
