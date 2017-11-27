
mod tui;
use ctrl::tui::Tui;

use mpsc::{self};

pub enum Alive {
    PathNr { nr: usize},
    Net
}

pub struct NetStats {
    index : usize,
    max   : usize
}


pub enum ReceiveDialog {
    PathNr { nr : usize},
    Debug,
    Host,
    Alive,
    NetStats
}


pub enum UiMsg {
    Update(ReceiveDialog,String)
}

pub enum SystemMsg {
    Update(ReceiveDialog,String)
}


pub struct Ctrl {
    rx: mpsc::Receiver<SystemMsg>,
    ui: Tui,
}





impl Ctrl {
    /// Create a new controller
    pub fn new(title: String, pathes: &Vec<String>, receiver: mpsc::Receiver<SystemMsg>, sender: mpsc::Sender<SystemMsg>, with_net: bool) -> Result<Ctrl, String> {
        Ok(Ctrl {
            rx: receiver,
            ui: Tui::new(title,sender.clone(), &pathes, with_net), 
        })
    }
    /// Run the controller
    pub fn run(&mut self) {
        while self.ui.step() {
            while let Some(message) = self.rx.try_iter().next() {
                // Handle messages arriving from the UI.
                match message {
                    SystemMsg::Update(recv_dialog,text) => {
                        self.ui
                            .ui_sender
                            .send(UiMsg::Update(recv_dialog,text))
                            .unwrap();
                    }
                };
            }
        }
    }
} // impl Controller
