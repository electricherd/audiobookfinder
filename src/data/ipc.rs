//! IPC module will hold all massive (that is why IPC) internal messages
//! which occur due to data collection, its start and its end.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum IPC {
    DoneSearching(u32),
}
