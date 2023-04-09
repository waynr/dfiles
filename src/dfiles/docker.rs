use std::process::Command;

pub fn run(args: Vec<String>) {
    let cmdstr: String = args.join(" ");
    log::debug!("docker run {}", cmdstr);

    let mut child = Command::new("docker")
        .arg("run")
        .args(args)
        .spawn()
        .expect("meow");

    let _ = child.wait().expect("failed waiting for child process");
}
