use std::{
    collections::HashMap,
    path::Path,
    sync::mpsc::{self, Receiver},
};

use rbx_dom_weak::{RbxTree, RbxInstanceProperties};
use log::info;

use crate::{
    place_runner::{PlaceRunner, PlaceRunnerOptions, open_rbx_place_file},
    message_receiver::RobloxMessage,
};

pub const DEFAULT_PORT: u16 = 54023;
pub const DEFAULT_TIMEOUT: u16 = 15;

pub fn run_place(
    path: &Path,
    extension: &str,
    options: PlaceRunnerOptions,
) -> Receiver<Option<RobloxMessage>> {
    let tree = open_rbx_place_file(path, extension);

    let place = PlaceRunner::new(tree, options);
    let (message_tx, message_rx) = mpsc::channel();

    place.run_with_sender(message_tx);

    message_rx
}

pub fn run_model(_path: &Path, _extension: &str) -> Receiver<Option<RobloxMessage>> {
    unimplemented!("Models are not yet supported by run-in-roblox.");
}

pub fn run_script(options: PlaceRunnerOptions) -> Receiver<Option<RobloxMessage>> {
    let tree = RbxTree::new(RbxInstanceProperties {
        name: String::from("Place"),
        class_name: String::from("DataModel"),
        properties: HashMap::new(),
    });

    let place = PlaceRunner::new(tree, options);
    let (message_tx, message_rx) = mpsc::channel();

    info!("Running place...");
    place.run_with_sender(message_tx);

    message_rx
}
