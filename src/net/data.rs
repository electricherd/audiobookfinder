//! The data view of the ssh communication, exchange between mainly client and server.

type VersionType = [u8; 3];

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DataAuth {
    id: String,
    version: VersionType,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum DataSession {
    Auth { auth : DataAuth},
    Data { dummy: String },
}

impl DataAuth {
    pub fn new(id: String) -> DataAuth {
        let major: u8 = str::parse(env!("CARGO_PKG_VERSION_MAJOR")).unwrap_or(0);
        let minor: u8 = str::parse(env!("CARGO_PKG_VERSION_MINOR")).unwrap_or(0);
        let patched: u8 = str::parse(env!("CARGO_PKG_VERSION_PATCH")).unwrap_or(0);
        let computed_version = [major, minor, patched];
        DataAuth {
            id: id,
            version: computed_version,
        }
    }
    pub fn get_id(&self) -> &String {
        &self.id
    }
    pub fn get_version(&self) -> &VersionType {
        &self.version
    }
}
