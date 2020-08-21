///! Definition and description of the yet output json format to webui
///
use super::{
    super::super::{
        ctrl::{ForwardNetMessage, NetMessages, Status},
        net::peer_representation,
    },
    CollectionPathAlive, InternalUiMsg,
};
use serde_json;
use std::fmt;

/// some internal shall fail because they are not supposed to be sent out,
/// so it results in a correct MyJson or the attribute that shall
/// not be send out but was tried to do ...
pub fn convert_internal_message(internal_msg: &InternalUiMsg) -> Result<WSJsonOut, String> {
    match internal_msg {
        InternalUiMsg::Update(ref forward_net_message) => match forward_net_message {
            ForwardNetMessage::Stats(message) => match message {
                NetMessages::Debug(_text) => Err("No debug messages".to_string()),
                NetMessages::ShowStats { show: _stats } => Ok(WSJsonOut::nothing()),
            },
            ForwardNetMessage::Add(ui_peer_to_add) => {
                Ok(WSJsonOut::update(NetData::add(PeerJson {
                    id: peer_representation::peer_to_hash_string(&ui_peer_to_add.id),
                    addr: ui_peer_to_add.addresses.clone(),
                })))
            }
            ForwardNetMessage::Delete(ui_peer_id_to_add) => Ok(WSJsonOut::update(NetData::remove(
                peer_representation::peer_to_hash_string(ui_peer_id_to_add),
            ))),
        },
        InternalUiMsg::StartAnimate(paths_alive, status) => match paths_alive {
            CollectionPathAlive::BusyPath(nr) => Ok(WSJsonOut::searching(AnimateData::cnt(
                RefreshData::path { nr: *nr },
                match status {
                    Status::ON => true,
                    Status::OFF => false,
                },
            ))),
            CollectionPathAlive::HostSearch => Ok(WSJsonOut::searching(AnimateData::cnt(
                RefreshData::net,
                match status {
                    Status::ON => true,
                    Status::OFF => false,
                },
            ))),
        },
        InternalUiMsg::StepAndAnimate(paths_alive) => match paths_alive {
            CollectionPathAlive::BusyPath(nr) => {
                Ok(WSJsonOut::refresh(RefreshData::path { nr: *nr }))
            }
            CollectionPathAlive::HostSearch => Ok(WSJsonOut::refresh(RefreshData::net)),
        },
        InternalUiMsg::PeerSearchFinished(peer, count) => {
            Ok(WSJsonOut::update(NetData::count(FinishPeer {
                peer: peer_representation::peer_to_hash_string(peer),
                count: *count,
            })))
        }
        InternalUiMsg::Terminate => Err("terminate is not really of interest, is it?".to_string()),
    }
}

pub fn convert_external_message(input_data: &str) -> Result<WSJsonIn, String> {
    serde_json::from_str(input_data).map_err(|_| match serde_json::from_str(input_data) {
        Ok::<serde_json::Value, serde_json::error::Error>(good_json) => format!(
            "error: this is a json but not as expected: {}",
            good_json.to_string()
        ),
        Err(_bad_json) => format!("error: this is not even a json: {}", input_data),
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

/// generate the REST dir paths to output json
pub fn rest_dirs(dirs: &Vec<String>) -> WSJsonOut {
    WSJsonOut::rest_dirs(dirs.to_vec())
}

//////////////////////////////////////////////////////////////////////
// help on that here https://serde.rs/enum-representations.html

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", content = "cnt")]
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

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PeerJson {
    id: String,
    addr: Vec<String>,
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FinishPeer {
    peer: String,
    count: u32,
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "view", content = "cnt")]
pub enum NetData {
    add(PeerJson),
    remove(String),
    count(FinishPeer),
}

// This is the critical part here, "event" and "data" to work with
// "ws_events_dispatcher.js" !!!!!!!!
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "event", content = "data")]
#[allow(non_camel_case_types)]
pub enum WSJsonOut {
    refresh(RefreshData),
    searching(AnimateData),
    init(InitData),
    update(NetData),
    rest_dirs(Vec<String>),
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
    rest_dir(String),
}
