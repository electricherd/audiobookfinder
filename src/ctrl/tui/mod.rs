//! The TUI parts, greatly using [Cursive](https://gyscos.github.io/Cursive/cursive/index.html)
//! showing table, the paths being searched, an alive for that, also the mDNS search performed
//! and later status of the connection to the found clients.
use super::super::{
    config,
    ctrl::{CollectionPathAlive, InternalUiMsg, NetMessages, Status},
};
use async_std::task;
use cursive::{
    align,
    traits::*, //{Identifiable,select};
    views::{Dialog, Layer, LinearLayout, ListView, Panel, ResizedView, TextView},
    Cursive,
};
use std::{
    iter::Iterator,
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

pub struct Tui {
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
    paths: Vec<AliveState>,
}

// following the advice for Rust clever enums
// to say what you mean, even if boolean
enum AliveSym {
    GoOn,
    Stop,
}

static RECT: usize = 40;
static SEPARATOR: &str = "..";
static STR_ALIVE: [char; 6] = ['.', '|', '/', '-', '\\', '*']; // first char is start, last char is stop

static DEBUG_TEXT_ID: &str = "debug_info";
static VIEW_LIST_ID: &str = "hostlist";

static PATHS_PREFIX_ID: &str = "pf";

static ID_HOST_INDEX: &str = "id_host";
static ID_HOST_NUMBER: &str = "id_max";
static ID_HOST_ALIVE: &str = "id_host_alive";

impl Tui {
    pub fn new(title: String, paths: &Vec<String>, with_net: bool) -> Result<Self, String> {
        let later_handle = Self::build_tui(title, paths, with_net)?;

        // now build the actual TUI object
        let mut tui = Tui {
            handle: later_handle,
            alive: AliveDisplayData {
                host: AliveState {
                    draw_char: 0,
                    runs: false,
                },
                paths: vec![
                    AliveState {
                        draw_char: 0,
                        runs: false,
                    };
                    paths.len()
                ],
            },
        };
        // quit by 'q' key
        tui.handle.add_global_callback('q', |s| s.quit());
        Ok(tui)
    }

    pub async fn run(
        &mut self,
        tui_receiver: Receiver<InternalUiMsg>,
        tui_sender: Sender<InternalUiMsg>,
    ) -> Result<(), String> {
        // this is the own channel just for tui
        info!("run tui async ... yes");
        loop {
            self.run_cursive(&tui_sender, &tui_receiver).await;
        }
        Ok(())
    }

    async fn run_cursive(
        &mut self,
        tui_sender: &Sender<InternalUiMsg>,
        tui_receiver: &Receiver<InternalUiMsg>,
    ) -> Result<bool, ()> {
        if !self.handle.is_running() {
            error!("Cursive is not running!");
            return Err(());
        }

        //info!("cursive run started");
        self.handle.step();
        //info!("cursive run terminated");
        // it's done, so
        //Err(())

        while let Some(message) = tui_receiver.try_iter().next() {
            info!("meessage received");
            match message {
                InternalUiMsg::Update((recv_dialog, text)) => match recv_dialog {
                    NetMessages::ShowNewHost => {
                        if let Some(mut host_list) = self.handle.find_name::<ListView>(VIEW_LIST_ID)
                        {
                            host_list.add_child("", TextView::new(format!("{}", text)));
                        } else {
                            error!("View {} could not be found!", VIEW_LIST_ID);
                        }
                    }
                    NetMessages::ShowStats { show } => {
                        let output = self.handle.find_name::<TextView>(ID_HOST_INDEX);
                        if let Some(mut found) = output {
                            found.set_content(show.line.to_string());
                        } else {
                            error!("View {} could not be found!", ID_HOST_INDEX);
                        }
                    }
                    NetMessages::Debug => {
                        if let Some(mut found) = self.handle.find_name::<TextView>(DEBUG_TEXT_ID) {
                            found.set_content(text);
                        } else {
                            error!("Debug view {} could not be found!", DEBUG_TEXT_ID);
                        }
                    }
                },
                InternalUiMsg::Animate(signal, on_off) => {
                    let sender_clone = tui_sender.clone();
                    match on_off {
                        Status::ON => {
                            self.toggle_alive(signal.clone(), AliveSym::GoOn);
                            let (already_running, toggle, timeout_id): (bool, &mut bool, usize) =
                                match signal {
                                    CollectionPathAlive::BusyPath(nr) => (
                                        self.alive.paths[nr].runs,
                                        &mut self.alive.paths[nr].runs,
                                        nr + 1,
                                    ),
                                    CollectionPathAlive::HostSearch => {
                                        (self.alive.host.runs, &mut self.alive.host.runs, 0)
                                    }
                                };
                            info!("received path alive");
                            if !already_running {
                                *toggle = true;
                                // if timer not started, start
                                task::sleep(Duration::from_millis(config::tui::ALIVE_REFRESH))
                                    .await;
                                sender_clone
                                    .send(InternalUiMsg::TimeOut(signal.clone()))
                                    .unwrap();
                            }
                        }
                        Status::OFF => {
                            self.toggle_alive(signal.clone(), AliveSym::Stop);
                            let toggle: &mut bool = match signal {
                                CollectionPathAlive::BusyPath(nr) => &mut self.alive.paths[nr].runs,
                                CollectionPathAlive::HostSearch => &mut self.alive.host.runs,
                            };
                            *toggle = false;
                        }
                    }
                }
                InternalUiMsg::TimeOut(which) => {
                    let (continue_timeout, timeout_id) = match which {
                        CollectionPathAlive::BusyPath(nr) => (self.alive.paths[nr].runs, nr + 1),
                        CollectionPathAlive::HostSearch => (self.alive.host.runs, 0),
                    };
                    if continue_timeout {
                        self.toggle_alive(which.clone(), AliveSym::GoOn);
                        let sender_clone = tui_sender.clone();
                        info!("refresh path alive");
                        task::sleep(Duration::from_millis(config::tui::ALIVE_REFRESH)).await;
                        sender_clone
                            .send(InternalUiMsg::TimeOut(which))
                            .unwrap_or_else(|_| {
                                // nothing to be done, that timer is just at the end
                            });
                    }
                }
            }
        }
        Ok(true)
    }

    ///    # Example test
    ///  ```
    ///  use adbflib::ctrl::tui::Tui;
    ///
    ///  let boundary = 15;
    ///  let input_vec: Vec<String> =
    ///     vec!["The duck went swimming.".into(),
    ///         "A cool hat does not fit you.".into()];
    ///  let expected_output: Vec<String> =
    ///     vec!["The duc..mming.".into(), "A cool ..t you.".into()];
    ///  let output = adbflib::ctrl::tui::Tui::split_intelligently_ralign(&input_vec, boundary);
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
                let diff = max_len - SEPARATOR.chars().count();
                let real_middle = diff / 2;
                let offset = if real_middle * 2 < diff { 1 } else { 0 };

                let first_part = el.chars().take(real_middle + offset).collect::<String>();
                let last_part = el.chars().skip(real_len - real_middle).collect::<String>();

                out.push(format!("{}{}{}", first_part, SEPARATOR, last_part));
            }
        }
        out
    }

    fn toggle_alive(&mut self, signal: CollectionPathAlive, on: AliveSym) {
        let (view_name, counter) = match signal {
            CollectionPathAlive::HostSearch => {
                (ID_HOST_ALIVE.to_string(), &mut self.alive.host.draw_char)
            }
            CollectionPathAlive::BusyPath(nr) => (
                format!("{}{}", PATHS_PREFIX_ID, nr),
                &mut self.alive.paths[nr].draw_char,
            ),
        };
        let mut output = self.handle.find_name::<TextView>(&view_name);
        if let Some(ref mut found) = output {
            let char_idx_to_put = match on {
                AliveSym::GoOn => {
                    *counter = (*counter + 1) % (STR_ALIVE.len() - 2);
                    *counter + 1
                }
                AliveSym::Stop => {
                    STR_ALIVE.len() - 1 // the stop symbol's index
                }
            };
            let out = STR_ALIVE[char_idx_to_put];
            found.set_content(out.to_string());
        }
    }

    fn build_tui(title: String, paths: &Vec<String>, with_net: bool) -> Result<Cursive, String> {
        let later_handle = Cursive::default();

        let screen_size = later_handle.screen_size();

        let max_cols = screen_size.x / RECT;

        if max_cols == 0 {
            return Err("Not able to build tui.".to_string());
        }
        let rows = paths.len() / max_cols + 1;

        // 80% of rect size
        let max_table_width = RECT * 5 / 6;

        let mut vertical_layout = LinearLayout::vertical();

        // add host list on top
        if with_net {
            vertical_layout.add_child(
                Dialog::around(
                    LinearLayout::vertical()
                        .child(ResizedView::with_fixed_height(
                            10,
                            ListView::new()
                                .child("", TextView::new(""))
                                .with_name(VIEW_LIST_ID),
                        ))
                        .child(
                            LinearLayout::horizontal()
                                .child(
                                    TextView::new(format!("{}", STR_ALIVE[0]))
                                        .with_name(ID_HOST_ALIVE),
                                )
                                .child(TextView::new(format!(" Looking at ")))
                                .child(
                                    TextView::new(format!("{}", 0))
                                        .h_align(align::HAlign::Right)
                                        .with_name(ID_HOST_INDEX),
                                )
                                .child(TextView::new(format!("/")))
                                .child(
                                    TextView::new(format!("{}", 0))
                                        .h_align(align::HAlign::Left)
                                        .with_name(ID_HOST_NUMBER),
                                ),
                        ),
                )
                .title("Host list"),
            );
        }

        // draw all pathes
        for j in 0..rows {
            let mut horizontal_layout = LinearLayout::horizontal();

            let cols = if j < rows - 1 {
                max_cols
            } else {
                paths.len() % max_cols
            };

            for i in 0..cols {
                let my_number = j * max_cols + i; // j >= 1

                let differentiate_path = Self::split_intelligently_ralign(paths, max_table_width);
                let path_name = format!("{}{}", PATHS_PREFIX_ID, my_number);

                horizontal_layout.add_child(Panel::new(
                    LinearLayout::horizontal()
                        .child(
                            // and link with an id
                            TextView::new(format!("{}", STR_ALIVE[0])).with_name(path_name),
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
            TextView::new(format!("debug_text: {}", debug_text)).with_name(DEBUG_TEXT_ID),
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
        let expected_output: Vec<String> = vec!["Герман ..мецких".into(), "Его мат..) была".into()];
        assert!(equal_with_boundary(&input_vec, &expected_output, boundary));
    }
}
