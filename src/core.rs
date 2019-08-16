use std::{
    collections::HashMap,
    path::Path,
    process,
    sync::mpsc,
};

use rbx_dom_weak::{RbxTree, RbxInstanceProperties};
use log::{info, error};
use colored::Colorize;

use crate::{
    place_runner::{PlaceRunner, PlaceRunnerOptions, open_rbx_place_file},
    message_receiver::{RobloxMessage, OutputLevel},
};

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

pub fn run_place(path: &Path, extension: &str, options: PlaceRunnerOptions) {
    let tree = open_rbx_place_file(path, extension);

    let place = PlaceRunner::new(tree, options);
    let (message_tx, message_rx) = mpsc::channel();

    place.run_with_sender(message_tx);

    while let Some(message) = message_rx.recv().expect("Problem receiving message") {
        print_message(&message);
    }
}

pub fn run_model(_path: &Path, _extension: &str) {
    error!("Models are not yet supported by run-in-roblox.");
    process::exit(1);
}

pub fn run_script(options: PlaceRunnerOptions) {
    let tree = RbxTree::new(RbxInstanceProperties {
        name: String::from("Place"),
        class_name: String::from("DataModel"),
        properties: HashMap::new(),
    });

    let place = PlaceRunner::new(tree, options);
    let (message_tx, message_rx) = mpsc::channel();

    info!("Running place...");
    place.run_with_sender(message_tx);

    while let Some(message) = message_rx.recv().unwrap() {
        print_message(&message);
    }
}

pub fn bad_path(path: &Path) -> ! {
    error!("Type of path {} could not be detected.", path.display());
    error!("Valid extensions are .lua, .rbxm, .rbxmx, .rbxl, and .rbxlx.");
    process::exit(1);
}
