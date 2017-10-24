
pub use super::tui::Tui;
use tui::{SystemMsg,UiMsg};

use mpsc;

pub struct Ctrl {
    rx: mpsc::Receiver<SystemMsg>,
    ui: Tui,
}

impl Ctrl {
    /// Create a new controller
    pub fn new(pathes: &Vec<&str>) -> Result<Ctrl, String> {
        let (tx, rx) = mpsc::channel::<SystemMsg>();
        Ok(Ctrl {
            rx: rx,
            ui: Tui::new(tx.clone(), &pathes)
        })
    }
    /// Run the controller
    pub fn run(&mut self) {
        while self.ui.step() {
            while let Some(message) = self.rx.try_iter().next() {
                // Handle messages arriving from the UI.
                match message {
                    SystemMsg::Update(text) => {
                        self.ui
                            .ui_sender
                            .send(UiMsg::Update(text))
                            .unwrap();
                    }
                };
            }
        }
    }
} // impl Controller
