mod message_receiver;
mod place_runner;
mod plugin;
mod place;
mod core;

use std::{
    fs::{read_to_string},
    path::Path,
    process,
};

use log::error;
use clap::{App, Arg};
use colored::Colorize;

use crate::{
    place_runner::PlaceRunnerOptions,
    message_receiver::{RobloxMessage, OutputLevel},
    core::{run_place, run_model, run_script},
};

const DEFAULT_PORT: u16 = 54023;
const DEFAULT_TIMEOUT: u16 = 15;

fn print_message(message: &RobloxMessage) {
    match message {
        RobloxMessage::Output {level, body} => {
            println!("{}", match level {
                OutputLevel::Print => body.normal(),
                OutputLevel::Info => body.cyan(),
                OutputLevel::Warning => body.yellow(),
                OutputLevel::Error => body.red(),
            });
        },
    }
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

    let default_port = DEFAULT_PORT.to_string();
    let default_timeout = DEFAULT_TIMEOUT.to_string();

    let app = App::new("run-in-roblox")
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))

        .arg(Arg::with_name("PATH")
            .help("The place, model, or script file to run inside Roblox")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("script")
            .short("s")
            .help("The lua script file to be executed in the opened place")
            .takes_value(true)
            .required(false)
            .conflicts_with("execute"))
        .arg(Arg::with_name("execute")
            .short("e")
            .help("The lua string to execute")
            .takes_value(true)
            .required(false)
            .conflicts_with("script"))
        .arg(Arg::with_name("port")
            .short("p")
            .help("The port used by the local server")
            .takes_value(true)
            .default_value(&default_port))
        .arg(Arg::with_name("timeout")
            .short("t")
            .help("The maximum time in seconds that the script can run")
            .takes_value(true)
            .default_value(&default_timeout));


    let matches = app.get_matches();

    let input = Path::new(matches.value_of("PATH").unwrap());

    let port = match matches.value_of("port") {
        Some(port) => port.parse::<u16>().expect("port must be an unsigned integer"),
        None => DEFAULT_PORT,
    };

    let timeout = match matches.value_of("timeout") {
        Some(timeout) => timeout.parse::<u16>().expect("timeout must be an unsigned integer"),
        None => DEFAULT_TIMEOUT,
    };

    let extension = match input.extension() {
        Some(e) => e.to_str().unwrap(),
        None => bad_path(input),
    };

    let messages = match extension {
        "lua" => {
            if let Some(_) = matches.value_of("script") {
                panic!("Cannot provide script argument when running a script file (remove `--script LUA_FILE_PATH`)")
            };
            if let Some(_) = matches.value_of("execute") {
                panic!("Cannot provide execute argument when running a script file (remove `--execute LUA_CODE`)")
            };

            let lua_script = read_to_string(input)
                    .expect("Something went wrong reading the file");

            run_script(PlaceRunnerOptions {
                port,
                timeout,
                lua_script: lua_script.as_ref(),
            })
        },
        "rbxm" | "rbxmx" => run_model(input, extension),
        "rbxl" | "rbxlx" => {
            let lua_script = if let Some(script_file_path) = matches.value_of("script") {
                read_to_string(script_file_path)
                    .expect("Something went wrong reading the file")
            } else if let Some(lua_string) = matches.value_of("execute") {
                lua_string.to_owned()
            } else {
                panic!("Lua code not provided (use `--script LUA_FILE_PATH` or `--execute LUA_CODE`)")
            };

            run_place(input, extension, PlaceRunnerOptions {
                port,
                timeout,
                lua_script: lua_script.as_ref(),
            })
        },
        _ => bad_path(input),
    };

    while let Some(message) = messages.recv().expect("Problem receiving message") {
        print_message(&message);
    }
}
