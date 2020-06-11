//! The ssh communication server, decisions will be made here, of how to interact with
//! the clients, is basically still taken from trussh example (with corrections)
//use super::super::data::DataSession;
use libp2p::PeerId;

#[derive(Clone)]
pub struct ComServer {
    pub peer_id: PeerId,
}

// fn data(
//     self,
//     channel: ChannelId,
//     data: &[u8],
//     mut session: server::Session,
// ) -> Self::FutureUnit {
//     let session_data: Result<DataSession, _> = bincode::deserialize_from(data);
//     match session_data {
//         Ok(deserialized) => match deserialized {
//             DataSession::Auth { ref auth } => {
//                 let client_id_string = format!("{:?}", auth.get_id());
//                 let client_version = auth.get_version();
//                 info!(
//                     "Srv[{:?}]: auth from channel {:?}: with id {:?} and version {:?}",
//                     self.peer_id.to_string(),
//                     channel,
//                     client_id_string,
//                     client_version
//                 );
//             }
//             DataSession::Data { .. } => {
//                 info!(
//                     "Srv[{:?}]: data from channel {:?}: {:?}",
//                     self.peer_id.to_string(),
//                     channel,
//                     std::str::from_utf8(data)
//                 );
//             }
//         },
//         Err(_) => {
//             info!(
//                 "Srv[{:?}]: not valid session data on channel {:?}: {:?}",
//                 self.peer_id.to_string(),
//                 channel,
//                 std::str::from_utf8(data)
//             );
//         }
//     }
//     session.data(channel, None, data);
// }
