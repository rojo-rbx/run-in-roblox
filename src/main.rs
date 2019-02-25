mod roblox_install;
mod message_receiver;
mod old_stuff;

use std::{
    path::Path,
    process,
};

use log::error;
use clap::{App, Arg};

fn run_place(path: &Path) {
    unimplemented!();
}

fn run_model(path: &Path) {
    unimplemented!();
}

fn run_script(path: &Path) {
    unimplemented!();
}

fn bad_path(path: &Path) -> ! {
    error!("Type of path {} could not be detected.", path.display());
    error!("Valid extensions are .lua, .rbxm, .rbxmx, .rbxl, and .rbxlx.");
    process::exit(1);
}

fn main() {
    {
        let log_env = env_logger::Env::default()
            .default_filter_or("warn");

        env_logger::Builder::from_env(log_env)
            .default_format_timestamp(false)
            .init();
    }

    let app = App::new("run-in-roblox")
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))

        .arg(Arg::with_name("PATH")
            .help("The place, model, or script file to run inside Roblox")
            .takes_value(true)
            .required(true));

    let matches = app.get_matches();

    let input = Path::new(matches.value_of("PATH").unwrap());

    let extension = match input.extension() {
        Some(e) => e.to_str().unwrap(),
        None => bad_path(input),
    };

    match extension {
        "lua" => run_script(input),
        "rbxm" | "rbxmx" => run_model(input),
        "rbxl" | "rbxlx" => run_place(input),
        _ => bad_path(input),
    }
}
