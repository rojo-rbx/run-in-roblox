use std::{
    collections::HashMap,
    io::{Write},
};

use rbx_xml::{EncodeError};

use rbx_dom_weak::{RbxValue, RbxTree, RbxInstanceProperties};

pub struct RunInRbxPlace {
    tree: RbxTree,
}

impl RunInRbxPlace {
    pub fn new(mut tree: RbxTree, port: u16) -> RunInRbxPlace {
        enable_http(&mut tree);
        add_plugin_marker(&mut tree, port);

        RunInRbxPlace {
            tree
        }
    }

    pub fn write<W: Write>(&self, output: W) -> Result<(), EncodeError> {
        let root_id = self.tree.get_root_id();
        let top_level_ids = self.tree.get_instance(root_id).unwrap().get_children_ids();

        rbx_xml::to_writer_default(output, &self.tree, top_level_ids)
    }
}

fn add_plugin_marker(tree: &mut RbxTree, port: u16) {
    let mut properties = HashMap::new();
    properties.insert(String::from("Value"), RbxValue::Int32 { value: port as i32 });

    let marker = RbxInstanceProperties {
        name: String::from("RUN_IN_ROBLOX_PORT"),
        class_name: String::from("IntValue"),
        properties,
    };

    let root_id = tree.get_root_id();
    tree.insert_instance(marker, root_id);
}

fn enable_http(tree: &mut RbxTree) {
    let http_service_id = match tree.descendants(tree.get_root_id())
        .find(|descendant| descendant.class_name == "HttpService") {
        Some(http_service) => Some(http_service.get_id()),
        None => None,
    };

    match http_service_id {
        Some(instance_id) => {
            let http_service = tree.get_instance_mut(instance_id)
                .expect("HttpService has disappeared suddenly");
            http_service.properties.entry("HttpEnabled".to_string())
                .or_insert(RbxValue::Bool { value : true });
        },
        None => insert_http_service(tree),
    }
}

fn insert_http_service(tree: &mut RbxTree) {
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

    tree.insert_instance(http_service, tree.get_root_id());
}
