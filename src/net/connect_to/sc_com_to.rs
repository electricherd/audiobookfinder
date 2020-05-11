//! The StateChart section to unravel especially net related async behavior,
//! using https://github.com/fitzgen/state_machine_future to start with.
//! https://crates.io/crates/extfsm looks also promising because it supposes to use
//! standard SCXML (https://en.wikipedia.org/wiki/SCXML), has Version 0.8 yet and seems to
//! be done on behalf of Juniper.
//! But unfortunately yet not well documented, so choosing first mentioned one
//! (which is documented excellently in https://github.com/fitzgen/state_machine_future), though
//! version is only 0.1.6 as of now, but what does it really mean?.

use futures::Poll;
use state_machine_future::RentToOwn;

#[derive(StateMachineFuture)]
pub enum SCClient {
    #[state_machine_future(start, transitions(Runner))]
    CreateAccordingIP {},

    #[state_machine_future(transitions(Finished, Runner))]
    Runner {},

    #[state_machine_future(ready)]
    Finished(()),

    #[state_machine_future(error)]
    Error(()),
}

impl PollSCClient for SCClient {
    fn poll_create_according_ip<'a>(
        create_according_ip: &'a mut RentToOwn<'a, CreateAccordingIP>,
    ) -> Poll<AfterCreateAccordingIP, ()> {
        let _input = create_according_ip.take();
        info!("connecting to client ...");
        let created_client = Runner {};
        transition!(created_client)
    }

    fn poll_runner<'a>(runner: &'a mut RentToOwn<'a, Runner>) -> Poll<AfterRunner, ()> {
        let _input = runner.take();
        info!("connecting to client and state chart...");
        transition!(Finished(()))
    }
}
