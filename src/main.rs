mod roblox_install;
mod message_receiver;
mod old_stuff;

use std::{
    collections::HashMap,
    fs::File,
    path::Path,
    process,
};

use rbx_dom_weak::{RbxTree, RbxInstanceProperties};
use log::error;
use clap::{App, Arg};

fn run_place(path: &Path, extension: &str) {
    let mut file = File::open(path)
        .expect("Couldn't open file");

    let mut tree = RbxTree::new(RbxInstanceProperties {
        name: String::from("Place"),
        class_name: String::from("DataModel"),
        properties: HashMap::new(),
    });
    let root_id = tree.get_root_id();

    match extension {
        "rbxl" => rbx_binary::decode(&mut tree, root_id, &mut file)
            .expect("Couldn't decode binary place file"),
        "rbxlx" => rbx_xml::decode(&mut tree, root_id, &mut file)
            .expect("Couldn't decode XML place file"),
        _ => unreachable!(),
    }

    // TODO: Actually use this place
}

fn run_model(_path: &Path, _extension: &str) {
    error!("Models are not yet supported by run-in-roblox.");
    process::exit(1);
}

fn run_script(_path: &Path) {
    error!("Scripts are not yet supported by run-in-roblox.");
    process::exit(1);
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
        "rbxm" | "rbxmx" => run_model(input, extension),
        "rbxl" | "rbxlx" => run_place(input, extension),
        _ => bad_path(input),
    }
}
