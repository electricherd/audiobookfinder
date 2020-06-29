///! Definition and description of the yet output json format to webui
///
use super::{super::super::ctrl::Status, CollectionPathAlive, InternalUiMsg};
use serde_json;
use std::fmt;

/// some internal shall fail because they are not supposed to be sent out,
/// so it results in a correct MyJson or the attribute that shall
/// not be send out but was tried to do ...
pub fn convert_internal_message(internal_msg: &InternalUiMsg) -> Result<WSJsonOut, String> {
    let ret = match internal_msg {
        InternalUiMsg::Update(_forward_net_message) => WSJsonOut::nothing(),
        InternalUiMsg::StartAnimate(paths_alive, status) => match paths_alive {
            CollectionPathAlive::BusyPath(nr) => WSJsonOut::searchPath(AnimateData::cnt(
                RefreshData::path { nr: *nr },
                match status {
                    Status::ON => true,
                    Status::OFF => false,
                },
            )),
            CollectionPathAlive::HostSearch => WSJsonOut::searchPath(AnimateData::cnt(
                RefreshData::net,
                match status {
                    Status::ON => true,
                    Status::OFF => false,
                },
            )),
        },
        InternalUiMsg::StepAndAnimate(paths_alive) => match paths_alive {
            CollectionPathAlive::BusyPath(nr) => WSJsonOut::refresh(RefreshData::path { nr: *nr }),
            CollectionPathAlive::HostSearch => WSJsonOut::refresh(RefreshData::net),
        },
    };
    Ok(ret)
}

pub fn convert_external_message(input_data: &str) -> Result<WSJsonIn, String> {
    serde_json::from_str(input_data).map_err(|_| match serde_json::from_str(input_data) {
        Ok::<serde_json::Value, serde_json::error::Error>(good_json) => format!(
            "error: this is a json but not as expected: {}",
            good_json.to_string()
        ),
        Err(bad_json) => format!("error: this is not even a json: {}", input_data),
    })
}

pub fn generate_init_data(paths: &Vec<String>) -> WSJsonOut {
    WSJsonOut::init(InitData {
        paths: paths
            .iter()
            .enumerate()
            .map(|(e, b)| PathData {
                nr: e.clone(),
                name: b.clone(),
            })
            .collect(),
    })
}

//////////////////////////////////////////////////////////////////////
// help on that here https://serde.rs/enum-representations.html

// todo: recreate InternalUiMessages as well to use more structs rather than enums, just like here

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum RefreshData {
    path { nr: usize },
    net,
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum AnimateData {
    cnt(RefreshData, bool),
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PathData {
    nr: usize,
    name: String,
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InitData {
    paths: Vec<PathData>,
}

// This is the critical part here, "event" and "data" to work with
// "ws_events_dispatcher.js" !!!!!!!!
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "event", content = "data")]
#[allow(non_camel_case_types)]
pub enum WSJsonOut {
    refresh(RefreshData),
    searchPath(AnimateData),
    init(InitData),
    nothing(),
}

impl fmt::Display for WSJsonOut {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

// This is the critical part here, "event" and "data" to work with
// "ws_events_dispatcher.js" !!!!!!!!
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "event", content = "data")]
#[allow(non_camel_case_types)]
pub enum WSJsonIn {
    start,
}
