// include the tui part :-)
pub mod tui;  // todo: pub is not recommended, I use it for doctest
use ctrl::tui::Tui;

use std::sync::mpsc;

#[derive(Clone)]
pub enum Alive {
    BusyPath(usize),
    HostSearch,
}

pub enum Status {
    ON,
    OFF,
}

pub struct NetStats {
    pub line: usize,
    pub max: usize,
}

pub enum ReceiveDialog {
    ShowNewPath { nr: usize },
    Debug,
    ShowNewHost,
    ShowStats { show: NetStats },
}

pub enum UiMsg {
    Update(ReceiveDialog, String),
    Animate(Alive, Status),
    TimeOut(Alive),
}

pub enum SystemMsg {
    Update(ReceiveDialog, String),
    StartAnimation(Alive, Status),
}

pub struct Ctrl {
    rx: mpsc::Receiver<SystemMsg>,
    ui: Tui,
}

impl Ctrl {
    /// Create a new controller
    pub fn new(
        title: String,
        paths: &Vec<String>,
        receiver: mpsc::Receiver<SystemMsg>,
        sender: mpsc::Sender<SystemMsg>,
        with_net: bool,
    ) -> Result<Ctrl, String> {
        Ok(Ctrl {
            rx: receiver,
            ui: Tui::new(title, sender.clone(), &paths, with_net),
        })
    }
    /// Run the controller
    pub fn run(&mut self) {
        while self.ui.step() {
            while let Some(message) = self.rx.try_iter().next() {
                // Handle messages arriving from the UI.
                match message {
                    SystemMsg::Update(recv_dialog, text) => {
                        self.ui
                            .ui_sender
                            .send(UiMsg::Update(recv_dialog, text))
                            .unwrap();
                    }
                    SystemMsg::StartAnimation(signal, on_off) => {
                        self.ui
                            .ui_sender
                            .send(UiMsg::Animate(signal, on_off))
                            .unwrap();
                    }
                };
            }
        }
    }
} // impl Controller
