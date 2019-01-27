//! The ctrl module should be the general controller of the program.
//! Right now, most controlling is in the net module, and here
//! is only a light facade to the tui messages.

pub mod tui; // todo: pub is not recommended, I use it for doctest
mod webui;

use std::sync::mpsc;
use std::thread;
use uuid::Uuid;

use self::tui::Tui;
use self::webui::WebUI;
use config;

#[derive(Clone)]
pub enum Alive {
    BusyPath(usize),
    HostSearch,
}

pub enum Status {
    ON,
    OFF,
}

pub enum ReceiveDialog {
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

pub struct NetStats {
    pub line: usize,
    pub max: usize,
}

pub struct Ctrl {
    rx: mpsc::Receiver<SystemMsg>,
    ui: Tui,
    uuid: Uuid,
    with_net: bool,
}

impl Ctrl {
    /// Create a new controller if everything fits.
    ///
    /// # Arguments
    /// * 'uuid' - The uuid this client/server uses
    /// * 'paths' - The paths that will be searched
    /// * 'receiver' - The receiver that takes incoming ctrl messages
    /// * 'sender'   - The sender that sends from ctrl
    /// * 'with_net' - If ctrl should consider net messages
    pub fn new_tui(
        uuid: Uuid,
        paths: &Vec<String>,
        receiver: mpsc::Receiver<SystemMsg>,
        sender: mpsc::Sender<SystemMsg>,
        with_net: bool,
    ) -> Result<Ctrl, String> {
        let c_ui = Tui::new(uuid.to_string(), sender.clone(), &paths, with_net)?;

        Ok(Ctrl {
            rx: receiver,
            ui: c_ui,
            uuid: uuid,
            with_net: with_net,
        })
    }
    /// Run the controller
    pub fn run_tui(&mut self) {
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

    /// Run the controller
    pub fn run_webui(&mut self) {
        let uuid_copy = self.uuid;
        let net_support = self.with_net;
        if webbrowser::open(&["http://", config::net::WEBSOCKET_ADDR].concat()).is_ok() {
            let _webui_runner = thread::spawn(move || {
                let _ = WebUI::new(uuid_copy, net_support);
            });
        } else {
            // Todo: debug
        }
    }
} // impl Controller
