///! Definition and description of the yet output json format to webui
///
use super::{super::super::ctrl::Status, CollectionPathAlive, InternalUiMsg};
use serde_json;
use std::fmt;

/// some internal shall fail because they are not supposed to be sent out,
/// so it results in a correct MyJson or the attribute that shall
/// not be send out put was tried to do ...
pub fn convert(internal_msg: &InternalUiMsg) -> Result<WSJson, String> {
    let ret = match internal_msg {
        InternalUiMsg::Update(_forward_net_message) => WSJson::nothing(),
        InternalUiMsg::StartAnimate(paths_alive, status) => match paths_alive {
            CollectionPathAlive::BusyPath(nr) => WSJson::animate(AnimateData::cnt(
                RefreshData::path { nr: *nr },
                match status {
                    Status::ON => true,
                    Status::OFF => false,
                },
            )),
            CollectionPathAlive::HostSearch => WSJson::animate(AnimateData::cnt(
                RefreshData::net,
                match status {
                    Status::ON => true,
                    Status::OFF => false,
                },
            )),
        },
        InternalUiMsg::StepAndAnimate(paths_alive) => match paths_alive {
            CollectionPathAlive::BusyPath(nr) => WSJson::refresh(RefreshData::path { nr: *nr }),
            CollectionPathAlive::HostSearch => WSJson::refresh(RefreshData::net),
        },
    };
    Ok(ret)
}

// todo: recreate InternalUiMessages as well to use more structs rather than enums, just like here

// help on that here https://serde.rs/enum-representations.html

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(untagged)]
pub enum RefreshData {
    path { nr: usize },
    net,
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(untagged)]
pub enum AnimateData {
    cnt(RefreshData, bool),
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(tag = "event", content = "data")]
#[allow(non_camel_case_types)] // no tag info means externally tagged by name therefore camel_case
pub enum WSJson {
    refresh(RefreshData),
    animate(AnimateData),
    nothing(),
}

impl fmt::Display for WSJson {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}
