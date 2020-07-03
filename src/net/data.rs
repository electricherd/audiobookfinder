//! The data view of the ssh communication, exchange between mainly client and server.
use libp2p_core::PeerId;
use std::string::String;
use std::vec::Vec;

type VersionType = [u8; 3];

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DataAuth {
    id: Vec<u8>,
    version: VersionType,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum DataSession {
    Auth { auth: DataAuth },
    Data { dummy: String },
}

impl DataAuth {
    #[allow(dead_code)]
    pub fn new(peer_id: PeerId) -> DataAuth {
        let major: u8 = str::parse(env!("CARGO_PKG_VERSION_MAJOR")).unwrap_or(0);
        let minor: u8 = str::parse(env!("CARGO_PKG_VERSION_MINOR")).unwrap_or(0);
        let patched: u8 = str::parse(env!("CARGO_PKG_VERSION_PATCH")).unwrap_or(0);
        let computed_version = [major, minor, patched];
        DataAuth {
            id: peer_id.into_bytes(),
            version: computed_version,
        }
    }

    #[allow(dead_code)]
    pub fn get_id(&self) -> &Vec<u8> {
        &self.id
    }

    #[allow(dead_code)]
    pub fn get_version(&self) -> &VersionType {
        &self.version
    }
}
