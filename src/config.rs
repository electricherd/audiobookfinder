#[allow(non_camel_case_types)]
pub mod net {

    pub static MDNS_SERVICE_NAME: &str = "_http._tcp"; // "_tcp.local"
    pub static MDNS_REGISTER_NAME: &str = "adbf";
    pub static MDNS_PORT : u16 = 80;
    pub static MDNS_TIMEOUT_SEC : u16 = 3;

    pub static SSH_CLIENT_KEY_FILE : &str = "/home/pe/.ssh/id_ed25519_pkcs8";
    pub static SSH_CLIENT_USERNAME : & str = "pe";
    pub static SSH_CLIENT_AND_PORT : & str = "0.0.0.0:2222";
    pub static SSH_HOST_AND_PORT : & str = "127.0.0.1:2222";

    pub struct changeable {

    }
}
