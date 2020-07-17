#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum IPC {
    DoneSearching(u32),
}
