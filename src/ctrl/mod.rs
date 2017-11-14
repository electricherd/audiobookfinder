
mod tui;
use ctrl::tui::Tui;

use mpsc::{self};

pub enum ReceiveDialog {
    PathNr { nr : usize},
    Debug
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
    pub fn new(pathes: &Vec<String>, receiver: mpsc::Receiver<SystemMsg>, sender: mpsc::Sender<SystemMsg>) -> Result<Ctrl, String> {
        Ok(Ctrl {
            rx: receiver,
            ui: Tui::new(sender.clone(), &pathes)
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

    pub fn debug(&mut self, text: String) {
        self.ui
            .ui_sender
            .send(UiMsg::Update(ReceiveDialog::Debug,text))
            .unwrap();
    }

} // impl Controller
