//! The StateChart section to unravel especially net related async behavior,
//! using https://github.com/fitzgen/state_machine_future to start with.
//! https://crates.io/crates/extfsm looks also promising because it supposes to use
//! standard SCXML (https://en.wikipedia.org/wiki/SCXML), has Version 0.8 yet and seems to
//!  be done on behalf of! Juniper.
//! But unfortunately yet not well documented, so choosing first mentioned one
//! (which is documented excellently in https://github.com/fitzgen/state_machine_future), though
//! version is only 0.1.6 as of now, but what does it really mean?.

use futures::Poll;
use net::ssh_client::com_client::ComClient;
use state_machine_future::RentToOwn;
use std::{net::IpAddr, sync::Arc, time::Duration};
use thrussh;

#[derive(StateMachineFuture)]
pub enum SCClient {
    #[state_machine_future(start, transitions(Runner))]
    CreateAccordingIP { id: String, address: IpAddr },

    #[state_machine_future(transitions(Finished))]
    Runner {
        client: ComClient,
        config: Arc<thrussh::client::Config>,
        address: IpAddr,
    },

    #[state_machine_future(ready)]
    Finished(()),

    #[state_machine_future(error)]
    Error(()),
}

impl PollSCClient for SCClient {
    fn poll_create_according_ip<'a>(
        create_according_ip: &'a mut RentToOwn<'a, CreateAccordingIP>,
    ) -> Poll<AfterCreateAccordingIP, ()> {
        let input = create_according_ip.take();

        let mut config = thrussh::client::Config::default();
        config.connection_timeout = Some(Duration::from_secs(600));
        let config = Arc::new(config);
        let client = ComClient::new(input.id);

        let created_client = Runner {
            client: client,
            config: config,
            address: input.address,
        };
        transition!(created_client)
    }
    fn poll_runner<'a>(runner: &'a mut RentToOwn<'a, Runner>) -> Poll<AfterRunner, ()> {
        let input = runner.take();
        if input.client.run(input.config, input.address).is_err() {
            error!("SSH Client example not working!!!");
        }
        transition!(Finished(()))
    }
}
