//! The StateChart for the server

use futures::Poll;
use net::ssh_client::com_client::ComServer;
use state_machine_future::RentToOwn;
use std::{net::IpAddr, sync::Arc, time::Duration};

#[derive(StateMachineFuture)]
pub enum SCServer {
    #[state_machine_future(start, transitions(WaitForAnswer))]
    StartOnReceive { id: String, address: IpAddr },

    #[state_machine_future(transitions(Finished))]
    WaitForAnswer {
        client: ComClient,
        config: Arc<thrussh::client::Config>,
        address: IpAddr,
    },

    #[state_machine_future(ready)]
    Finished(()),

    #[state_machine_future(error)]
    Error(()),
}

impl PollSCServer for SCServer {
    fn poll_start_on_receive<'a>(
        start_on_receive: &'a mut RentToOwn<'a, StartOnReceive>,
    ) -> Poll<AfterStartOnReceive, ()> {
        let input = create_according_ip.take();

        let created_client = Runner {
            client: client,
            config: config,
            address: input.address,
        };
        transition!(created_client)
    }
    fn poll_wait_for_anwer<'a>(
        wait_for_answer: &'a mut RentToOwn<'a, Runner>,
    ) -> Poll<AfterWaitForAnswer, ()> {
        let input = wait_for_answer.take();
        info!("SSH Server wait for answer ...!!!");
        if input.client.run(input.config, input.address).is_err() {
            error!("SSH Server example not working!!!");
        }
        info!("SSH Server wait for answer finished!!!");
        transition!(Finished(()))
    }
}
