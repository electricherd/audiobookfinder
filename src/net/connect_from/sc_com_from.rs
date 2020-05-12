//! The StateChart for the server

use net::ssh_client::com_client::ComServer;
use smlang::statemachine;
use std::{net::IpAddr, sync::Arc, time::Duration};

pub struct StartData {
    id: String,
    address: IpAddr,
}

pub struct WaitForAnswerData {
    client: ComClient,
    config: Arc<thrussh::client::Config>,
    address: IpAddr,
}

statemachine! {
    *StartState + Event1(StartData) / start_on_receive = Waiting,
    Waiting + Event2(WaitForAnswerData) / wait_for_answer = StartState,
}

pub struct SCServer {}

impl StateMachineContext for SCServer {
    fn start_on_receive(&mut self, start_data: &StartEventData) -> WaitForAnswerData {
        info!("connecting to client ...");
        WaitForAnswerData {
            client: client,
            config: config,
            address: input.address,
        }
    }

    fn wait_for_answer(&mut self, wait_data: &WaitForAnswerData) {
        info!("SSH Server wait for answer ...!!!");
        if wait_data
            .client
            .run(wait_data.config, wait_data.address)
            .is_err()
        {
            error!("SSH Server example not working!!!");
        }
        info!("SSH Server wait for answer finished!!!");
    }
}
