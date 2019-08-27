use shiplift::BuildOptions;
use shiplift::Docker;
use shiplift::ContainerOptions;
use tokio::prelude::Future;
use tokio::prelude::Stream;
use serde::Deserialize;
use serde_json::from_value;

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

pub fn run(opts: &ContainerOptions) {
    let docker = Docker::new();

    let fut = docker
        .containers()
        .create(opts)
        .map(|info| println!("{:?}", info))
        .map_err(|e| eprintln!("{:?}", e));
    tokio::run(fut);
}
