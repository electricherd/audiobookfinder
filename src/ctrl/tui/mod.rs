use super::super::cursive::Cursive;
use super::super::cursive::align;
use super::super::cursive::views::{Dialog, Layer, LinearLayout, ListView, Panel, TextView};
use super::super::cursive::traits::*; //{Identifiable,select};

use std::iter::Iterator;
use std::thread;
use std::sync::mpsc;
use std::time::Duration;

use ctrl::{Alive, ReceiveDialog, Status, SystemMsg, UiMsg};
use common::ThreadPool;
use config;

pub struct Tui {
    pub ui_receiver: mpsc::Receiver<UiMsg>,
    pub ui_sender: mpsc::Sender<UiMsg>,
    pub system_sender: mpsc::Sender<SystemMsg>,

    handle: Cursive,
    alive: AliveDisplayData,
}

#[derive(Clone)]
struct AliveState {
    draw_char: usize,
    runs: bool,
}

struct AliveDisplayData {
    host: AliveState,
    pathes: Vec<AliveState>,
    timers: ThreadPool,
}

// following the advice for Rust clever enums
// to say what you mean, even if boolean
enum AliveSym {
    GoOn,
    Stop,
}

static RECT: usize = 40;
static SEPERATOR: &str = "..";
static STR_ALIVE: [char; 5] = ['*', '|', '/', '-', '\\']; // first char is start and not animated

static DEBUG_TEXT_ID: &str = "debug_info";
static VIEW_LIST_HOST: &str = "hostlist";

static PATHS_PREFIX_ID: &str = "pf";

static ID_HOST_INDEX: &str = "id_host";
static ID_HOST_NUMBER: &str = "id_max";
static ID_HOST_ALIVE: &str = "id_host_alive";

impl Tui {
    pub fn new(
        title: String,
        system: mpsc::Sender<SystemMsg>,
        pathes: &Vec<String>,
        with_net: bool,
    ) -> Result<Tui, String> {
        let later_handle = Self::build_tui(title, pathes, with_net)?;

        // now build the actual TUI object
        let (tui_sender, tui_receiver) = mpsc::channel::<UiMsg>();
        let mut tui = Tui {
            handle: later_handle,
            ui_sender: tui_sender,
            ui_receiver: tui_receiver,
            system_sender: system,
            alive: AliveDisplayData {
                host: AliveState {
                    draw_char: 0,
                    runs: false,
                },
                pathes: vec![
                    AliveState {
                        draw_char: 0,
                        runs: false,
                    };
                    pathes.len()
                ],
                timers: ThreadPool::new(pathes.len() + 1),
            },
        };

        // test this, to update every with 20fps / this should be done when something changes ..... grrrr
        tui.handle.set_fps(20);
        // quit by 'q' key
        tui.handle.add_global_callback('q', |s| s.quit());
        Ok(tui)
    }

    pub fn step(&mut self) -> bool {
        if !self.handle.is_running() {
            return false;
        }

        while let Some(message) = self.ui_receiver.try_iter().next() {
            match message {
                UiMsg::Update(recv_dialog, text) => {
                    match recv_dialog {
                        ReceiveDialog::ShowNewPath { nr } => {
                            if let Some(mut _found) = self.handle
                                .find_id::<TextView>(&format!("{}{}", PATHS_PREFIX_ID, nr))
                            {
                                //found.append_content(&text.clone());
                            }
                        }
                        ReceiveDialog::ShowNewHost => {
                            if let Some(mut host_list) =
                                self.handle.find_id::<ListView>(VIEW_LIST_HOST)
                            {
                                error!("{:?} should be drawn!!", text);
                                host_list.add_child("", TextView::new(format!("{}", text)));
                            } else {
                                error!("{:?} could not be drawn!!", text);
                            }
                        }
                        ReceiveDialog::ShowStats { show } => {
                            let output = self.handle.find_id::<TextView>(ID_HOST_INDEX);
                            if let Some(mut found) = output {
                                found.set_content(show.line.to_string());
                            }
                        }
                        ReceiveDialog::Debug => {
                            if let Some(mut found) = self.handle.find_id::<TextView>(DEBUG_TEXT_ID)
                            {
                                found.set_content(text);
                            }
                        }
                    }
                }
                UiMsg::Animate(signal, on_off) => {
                    let sender_clone = self.ui_sender.clone();
                    match on_off {
                        Status::ON => {
                            self.toggle_alive(signal.clone(), AliveSym::GoOn);
                            let (already_running, toggle,timeout_id): (
                                bool,
                                &mut bool,
                                usize
                            ) = match signal {
                                Alive::BusyPath(nr) => {
                                    (self.alive.pathes[nr].runs, &mut self.alive.pathes[nr].runs,nr+1)
                                }
                                Alive::HostSearch => {
                                    (self.alive.host.runs, &mut self.alive.host.runs,0)
                                }
                            };
                            if !already_running {
                                *toggle = true;
                                // if timer not started, start
                                self.alive.timers.renew(timeout_id, move || {
                                    thread::sleep(Duration::from_millis(
                                        config::tui::ALIVE_REFRESH,
                                    ));
                                    sender_clone.send(UiMsg::TimeOut(signal.clone())).unwrap();
                                });
                            }
                        }
                        Status::OFF => {
                            self.toggle_alive(signal.clone(), AliveSym::Stop);
                            let toggle: &mut bool = match signal {
                                Alive::BusyPath(nr) => &mut self.alive.pathes[nr].runs,
                                Alive::HostSearch => &mut self.alive.host.runs,
                            };
                            *toggle = false;
                        }
                    }
                }
                UiMsg::TimeOut(which) => {
                    let (continue_timeout, timeout_id) = match which {
                        Alive::BusyPath(nr) => (self.alive.pathes[nr].runs, nr + 1),
                        Alive::HostSearch => (self.alive.host.runs, 0),
                    };
                    if continue_timeout {
                        self.toggle_alive(which.clone(), AliveSym::GoOn);
                        let sender_clone = self.ui_sender.clone();
                        self.alive.timers.renew(timeout_id, move || {
                            thread::sleep(Duration::from_millis(config::tui::ALIVE_REFRESH));
                            sender_clone.send(UiMsg::TimeOut(which)).unwrap();
                        });
                    }
                }
            }
        }
        // step through the TUI
        self.handle.step();
        true
    }

    ///    # Example test
    ///  ```
    ///  use adfblib::ctrl::tui::Tui;
    ///
    ///  let boundary = 15;
    ///  let input_vec: Vec<String> =
    ///     vec!["The duck went swimming.".into(),
    ///         "A cool hat does not fit you.".into()];
    ///  let expected_output: Vec<String> =
    ///     vec!["The duc..mming.".into(), "A cool ..t you.".into()];
    ///  let output = adfblib::ctrl::tui::Tui::split_intelligently_ralign(&input_vec, boundary);
    ///  ```
    // todo: not public... only due to testing
    pub fn split_intelligently_ralign(vec: &Vec<String>, max_len: usize) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for el in vec {
            let real_len = el.chars().count();
            // simple case
            if real_len < max_len {
                out.push(format!(
                    "{}{}",
                    " ".repeat(max_len - real_len),
                    el.to_string()
                ));
            } else if real_len < max_len / 2 {
                out.push(el.chars().skip(real_len - max_len).collect::<String>());
            } else {
                let diff = max_len - SEPERATOR.chars().count();
                let real_middle = diff / 2;
                let offset = if real_middle * 2 < diff { 1 } else { 0 };

                let first_part = el.chars().take(real_middle + offset).collect::<String>();
                let last_part = el.chars().skip(real_len - real_middle).collect::<String>();

                out.push(format!("{}{}{}", first_part, SEPERATOR, last_part));
            }
        }
        out
    }

    fn toggle_alive(&mut self, signal: Alive, on: AliveSym) {
        let (view_name, counter) = match signal {
            Alive::HostSearch => (ID_HOST_ALIVE.to_string(), &mut self.alive.host.draw_char),
            Alive::BusyPath(nr) => (
                format!("{}{}", PATHS_PREFIX_ID, nr),
                &mut self.alive.pathes[nr].draw_char,
            ),
        };
        let mut output = self.handle.find_id::<TextView>(&view_name);
        if let Some(ref mut found) = output {
            let char_idx_to_put = match on {
                AliveSym::GoOn => {
                    *counter = (*counter + 1) % (STR_ALIVE.len() - 1);
                    *counter + 1
                }
                AliveSym::Stop => {
                    0 // the stop symbol's index
                }
            };
            let out = STR_ALIVE[char_idx_to_put];
            found.set_content(out.to_string());
        }
    }

    fn build_tui(title: String, pathes: &Vec<String>, with_net: bool) -> Result<Cursive, String> {
        let later_handle = Cursive::new();

        let screen_size = later_handle.screen_size();

        let max_cols = screen_size.x / RECT;

        if max_cols == 0 {
            return Err("Not able to build tui.".to_string());
        }
        let rows = pathes.len() / max_cols + 1;

        // 80% of rect size
        let max_table_width = RECT * 5 / 6;

        let mut vertical_layout = LinearLayout::vertical();

        // add host list on top
        if with_net {
            vertical_layout.add_child(
                Dialog::around(
                    LinearLayout::vertical()
                        .child(ListView::new().fixed_height(10).with_id(VIEW_LIST_HOST))
                        .child(
                            LinearLayout::horizontal()
                                .child(
                                    TextView::new(format!("{}", STR_ALIVE[0]))
                                        .with_id(ID_HOST_ALIVE),
                                )
                                .child(TextView::new(format!(" Looking at ")))
                                .child(
                                    TextView::new(format!("{}", 0))
                                        .h_align(align::HAlign::Right)
                                        .with_id(ID_HOST_INDEX),
                                )
                                .child(TextView::new(format!("/")))
                                .child(
                                    TextView::new(format!("{}", 0))
                                        .h_align(align::HAlign::Left)
                                        .with_id(ID_HOST_NUMBER),
                                ),
                        ),
                ).title("Host list"),
            );
        }

        // draw all pathes
        for j in 0..rows {
            let mut horizontal_layout = LinearLayout::horizontal();

            let cols = if j < rows - 1 {
                max_cols
            } else {
                pathes.len() % max_cols
            };

            for i in 0..cols {
                let my_number = j * max_cols + i; // j >= 1

                let differentiate_path = Self::split_intelligently_ralign(pathes, max_table_width);
                let path_name = format!("{}{}", PATHS_PREFIX_ID, my_number);

                horizontal_layout.add_child(Panel::new(
                    LinearLayout::horizontal()
                        .child(
                            // and link with an id
                            TextView::new(format!("{}", STR_ALIVE[0])).with_id(path_name),
                        )
                        .child(
                            TextView::new(format!("{}", differentiate_path[my_number]))
                                .fixed_height(3),
                        ),
                ));
            }
            vertical_layout.add_child(horizontal_layout);
        }

        // debug output here
        let debug_text = max_table_width;
        vertical_layout.add_child(Panel::new(
            TextView::new(format!("debug_text: {}", debug_text)).with_id(DEBUG_TEXT_ID),
        ));

        // return the dialog
        let layer =
            Dialog::around(Layer::new(vertical_layout)).title(format!("This uuid: {}", title));
        let mut later_handle = later_handle;
        later_handle.add_layer(layer);
        Ok(later_handle)
    }
} // impl Tui

#[cfg(test)]
mod tests {
    use super::*;

    //run "cargo test -- --nocapture" to see debug println
    fn equal_with_boundary(input: &Vec<String>, expected: &Vec<String>, boundary: usize) -> bool {
        let output = Tui::split_intelligently_ralign(&input, boundary);
        println!("|{:?}|", output);
        println!("|{:?}|", expected);
        output.len() == expected.len() && output.iter().zip(expected.iter()).all(|(e, o)| e == o)
    }

    #[test]
    fn split_intelligently_ralign_exists() {
        let boundary = 20;
        let input_vec: Vec<String> = Vec::new();
        let _ = Tui::split_intelligently_ralign(&input_vec, boundary);
        assert!(true);
    }

    #[test]
    fn split_intelligently_ralign_alignment() {
        let boundary = 15;
        let input_vec: Vec<String> = vec!["duck".into(), "monkey".into()];
        let expected_output: Vec<String> = vec!["           duck".into(), "         monkey".into()];
        assert!(equal_with_boundary(&input_vec, &expected_output, boundary));
    }

    #[test]
    fn split_intelligently_ralign_fail() {
        let boundary = 20;
        let input_vec: Vec<String> = vec!["duck".into(), "monkey".into()];
        let expected_output: Vec<String> = vec!["duckkk".into(), "monkeyyyy".into()];
        assert!(!equal_with_boundary(&input_vec, &expected_output, boundary));
    }

    #[test]
    fn split_intelligently_ralign_shorten() {
        let boundary = 15;
        let input_vec: Vec<String> = vec![
            "The duck went swimming.".into(),
            "A cool hat does not fit you.".into(),
        ];
        let expected_output: Vec<String> = vec!["The duc..mming.".into(), "A cool ..t you.".into()];
        assert!(equal_with_boundary(&input_vec, &expected_output, boundary));
    }

    #[test]
    fn split_intelligently_ralign_utf8_russian() {
        let boundary = 15;
        let input_vec: Vec<String> = vec![
            "Герман Гессе родился в семье немецких".into(),
            "Его мать Мария Гундерт (1842—1902) была".into(),
        ];
        let expected_output: Vec<String> = vec![
            "Герман ..мецких".into(),
            "Его мат..) была".into(),
        ];
        assert!(equal_with_boundary(&input_vec, &expected_output, boundary));
    }

}
