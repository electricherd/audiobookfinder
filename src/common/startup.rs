// to synchronize start of threads
use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    time::Duration,
};

const THREAD_SYNC_TIMEOUT: Duration = Duration::from_millis(100);

pub enum Process {
    Go,
}
pub enum SyncStartUp {
    SendReceiver(Sender<Process>),
    NoWait,
}

pub struct StartUp {}

impl StartUp {
    pub fn block_on_sync(ready_sender: Sender<SyncStartUp>, name: &str) {
        let (ready_sync_sender_from, ready_sync_receiver_from) = channel::<Process>();
        ready_sender
            .send(SyncStartUp::SendReceiver(ready_sync_sender_from))
            .expect(&["collection from ui receiver for", name, "not yet there???"].join(" "));
        trace!("sending out to sync ui start for {:?}", name);
        ready_sync_receiver_from
            .recv_timeout(THREAD_SYNC_TIMEOUT)
            .expect(&["the ui channel for", name, "was just sent?"].join(" "));
        trace!("sync ui start received for {:?}", name);
    }

    // todo: timeout and results should be considered!!! a little panicking maybe
    pub fn send_and_block2(
        ready_1st_receiver: &Receiver<SyncStartUp>,
        ready_2nd_receiver: &Receiver<SyncStartUp>,
    ) {
        // a yet not so nice implementation of select! for 2 threads
        loop {
            if Self::timeout_try_2_receiver((ready_1st_receiver, ready_2nd_receiver)) {
                break;
            } else {
                if Self::timeout_try_2_receiver((ready_2nd_receiver, &ready_1st_receiver)) {
                    break;
                }
            }
        }
    }

    // todo: timeout and results should be considered!!! a little panicking maybe
    fn send_back_return(all: Vec<&SyncStartUp>) {
        for el in all {
            match el {
                SyncStartUp::SendReceiver(sender) => {
                    sender
                        .send(Process::Go)
                        .expect("there has to be a receiver, the sender was sent here!");
                }
                SyncStartUp::NoWait => {}
            }
        }
    }

    fn timeout_try_2_receiver((a, b): (&Receiver<SyncStartUp>, &Receiver<SyncStartUp>)) -> bool {
        if let Ok(try_receiver1) = a.try_recv() {
            trace!("sync with 1st worked");
            b.recv_timeout(THREAD_SYNC_TIMEOUT)
                .and_then(|in_time_receiver2| {
                    trace!("sync with 2nd worked");
                    Self::send_back_return(vec![&try_receiver1, &in_time_receiver2]);
                    Ok(true)
                })
                .unwrap_or(false)
        } else {
            false
        }
    }
}
