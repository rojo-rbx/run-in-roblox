//! This file should slowly be factored away into smaller pieces. It came from
//! rbx-dom's generate_rbx_reflection crate.

use std::{
    collections::HashMap,
    fs::{self, File},
    process::{self, Command},
    str,
    time::Duration,
};

use rbx_dom_weak::{RbxValue, RbxTree, RbxInstanceProperties};
use tempfile::tempdir;

use crate::{
    roblox_install::RobloxStudio,
    message_receiver::{Message, RobloxMessage, MessageReceiver, MessageReceiverOptions},
};

const PORT: u16 = 54023;

static PLUGIN_TEMPLATE: &'static str = include_str!("plugin_main_template.lua");

/// A wrapper for process::Child that force-kills the process on drop.
struct KillOnDrop(process::Child);

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let _dont_care = self.0.kill();
    }
}

fn create_place() -> RbxTree {
    let mut tree = RbxTree::new(RbxInstanceProperties {
        name: String::from("run_in_roblox Place"),
        class_name: String::from("DataModel"),
        properties: HashMap::new(),
    });

    let root_id = tree.get_root_id();

    let http_service = RbxInstanceProperties {
        name: String::from("HttpService"),
        class_name: String::from("HttpService"),
        properties: {
            let mut properties = HashMap::new();

            properties.insert(
                String::from("HttpEnabled"),
                RbxValue::Bool { value: true },
            );

            properties
        },
    };
    tree.insert_instance(http_service, root_id);

    let marker = RbxInstanceProperties {
        name: String::from("RUN_IN_ROBLOX_MARKER"),
        class_name: String::from("StringValue"),
        properties: HashMap::new(),
    };
    tree.insert_instance(marker, root_id);

    tree
}

pub fn inject_plugin_main(tree: &mut RbxTree) {
    let complete_source = PLUGIN_TEMPLATE
        .replace("{{PORT}}", &PORT.to_string());

    let entry_point = RbxInstanceProperties {
        name: String::from("generate_rbx_reflection main"),
        class_name: String::from("Script"),
        properties: {
            let mut properties = HashMap::new();

            properties.insert(
                String::from("Source"),
                RbxValue::String { value: complete_source },
            );

            properties
        },
    };

    let root_id = tree.get_root_id();
    tree.insert_instance(entry_point, root_id);
}

pub fn run_in_roblox(plugin: &RbxTree) -> Vec<RobloxMessage> {
    let studio_install = RobloxStudio::locate()
        .expect("Could not find Roblox Studio installation");

    let work_dir = tempdir()
        .expect("Could not create temporary directory");

    let place_file_path = work_dir.path().join("place.rbxlx");
    let plugin_file_path = studio_install.built_in_plugins_path().join("run_in_roblox.rbxmx");

    {
        let place = create_place();
        let mut place_file = File::create(&place_file_path)
            .expect("Could not create temporary place file");

        let root_id = place.get_root_id();
        let top_level_ids = place.get_instance(root_id).unwrap().get_children_ids();

        rbx_xml::encode(&place, top_level_ids, &mut place_file)
            .expect("Could not serialize temporary place file to disk");
    }

    {
        let mut plugin_file = File::create(&plugin_file_path)
            .expect("Could not create temporary plugin file");

        let root_id = plugin.get_root_id();

        rbx_xml::encode(&plugin, &[root_id], &mut plugin_file)
            .expect("Could not serialize plugin file to disk");
    }

    let message_receiver = MessageReceiver::start(MessageReceiverOptions {
        port: PORT,
    });

    let _studio_process = KillOnDrop(Command::new(studio_install.exe_path())
        .arg(format!("{}", place_file_path.display()))
        .spawn()
        .expect("Couldn't start Roblox Studio"));

    match message_receiver.recv_timeout(Duration::from_secs(10)).expect("Timeout reached") {
        Message::Start => {},
        _ => panic!("Invalid first message received"),
    }

    let mut messages = Vec::new();

    loop {
        let message = message_receiver.recv();

        match message {
            Message::Start => {},
            Message::Stop => break,
            Message::Messages(mut roblox_messages) => roblox_messages.drain(..)
                .for_each(|message| messages.push(message)),
        }
    }

    message_receiver.stop();
    let _dont_care = fs::remove_file(&plugin_file_path);

    messages
}
