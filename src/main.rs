mod dfiles;

use clap::App;
use clap::Arg;
use clap::SubCommand;

fn main() {
    let dirname = Arg::with_name("dirname")
        .short("d")
        .default_value("/home/wayne/projects/dockerfiles")
        .long("dockerfiles_dir")
        .takes_value(true)
        .value_name("DIR");

    let matches = App::new("dfiles")
        .version("0.0")
        .arg(dirname)
        .subcommand(SubCommand::with_name("run"))
        .subcommand(SubCommand::with_name("build"))
        .get_matches();

    let root_dir = std::path::Path::new(matches.value_of("dirname").unwrap());
    let chrome_dir = root_dir.join("chrome");

    match matches.subcommand() {
        ("run", _) => (),
        ("build", _) => dfiles::build::build(chrome_dir),
        (_, _) => { 
            eprintln!("unsupported subcommand");
            std::process::exit(1);
        },
    }
}
