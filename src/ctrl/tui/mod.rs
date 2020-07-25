//! The TUI parts, greatly using [Cursive](https://gyscos.github.io/Cursive/cursive/index.html)
//! showing table, the paths being searched, an alive for that, also the mDNS search performed
//! and later status of the connection to the found clients.

use super::super::{
    config,
    ctrl::{CollectionPathAlive, ForwardNetMessage, InternalUiMsg, NetMessages, Status},
    net::peer_representation,
};

use async_std::task;
use cursive::{
    align,
    traits::*, //{Identifiable,select};
    views::{
        Dialog, Layer, LinearLayout, ListChild, ListView, Panel, ResizedView, TextContent, TextView,
    },
    Cursive,
    CursiveExt,
};
use std::{
    iter::Iterator,
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

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

/// The Tui wrapper holds cursive tui and the
/// alive data of the ui-state needed.
pub struct Tui {
    handle: Cursive,
    alive: AliveDisplayData,
}

impl Tui {
    /// Steps through cursive tui and if a message is
    /// received, it will:
    /// - update the ui (net and path)
    /// - re-send a timed animation change
    ///
    /// # Arguments
    ///
    /// * `title` - the title of the tui
    /// * `paths` - the paths to be shown searching bar and name them
    /// * `with_net` - if net feature is activated for displaying live data
    pub fn new(title: String, paths: &Vec<String>, with_net: bool) -> Result<Self, String> {
        let later_handle = Self::build_tui(title, paths, with_net)?;

        // now build the actual TUI object
        let mut tui = Self {
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

    /// Refreshes the screen and returns
    /// if tui is shut down (e.g. due to 'q' quitting)
    pub async fn refresh(&mut self) -> bool {
        if !self.handle.is_running() {
            trace!("Cursive is not running!");
            false
        } else {
            self.handle.refresh();
            true
        }
    }

    /// Steps through cursive tui and if a message is
    /// received, it will:
    /// - update the ui (net and path)
    /// - re-send a timed animation change
    ///
    /// # Arguments
    ///
    /// * `tui_sender` - the sender to re-send messages
    /// * `tui_receiver` - the receiver to listen on
    pub async fn run_cursive(
        &mut self,
        tui_sender: &Sender<InternalUiMsg>,
        tui_receiver: &Receiver<InternalUiMsg>,
    ) {
        // step ui
        self.handle.step();

        if let Ok(message) = tui_receiver.try_recv() {
            match message {
                InternalUiMsg::Update(forward_net_message) => match forward_net_message {
                    ForwardNetMessage::Add(ui_peer) => {
                        if let Some(mut host_list) = self.handle.find_name::<ListView>(VIEW_LIST_ID)
                        {
                            let content = TextContent::new(
                                peer_representation::peer_to_hash_string(&ui_peer.id),
                            );
                            host_list.add_child("", TextView::new_with_content(content));
                        } else {
                            error!("View {} could not be found!", VIEW_LIST_ID);
                        }
                    }
                    ForwardNetMessage::Delete(ui_peer_to_delete) => {
                        if let Some(mut host_list) = self.handle.find_name::<ListView>(VIEW_LIST_ID)
                        {
                            for i in 0..host_list.len() {
                                let child = host_list.get_row(i);
                                match child {
                                    ListChild::Row(_label, ref view) => {
                                        // text if view is textview with content of peer
                                        if let Some(textview) = view.downcast_ref::<TextView>() {
                                            let found_text =
                                                textview.get_content().source().to_string();
                                            let search_text =
                                                peer_representation::peer_to_hash_string(
                                                    &ui_peer_to_delete,
                                                );
                                            if found_text == search_text {
                                                host_list.remove_child(i);
                                                break;
                                            }
                                        }
                                    }
                                    ListChild::Delimiter => (),
                                }
                            }
                        } else {
                            error!("View {} could not be found!", VIEW_LIST_ID);
                        }
                    }
                    ForwardNetMessage::Stats(net_message) => match net_message {
                        NetMessages::ShowStats { show } => {
                            let output = self.handle.find_name::<TextView>(ID_HOST_INDEX);
                            if let Some(mut found) = output {
                                found.set_content(show.line.to_string());
                            } else {
                                error!("View {} could not be found!", ID_HOST_INDEX);
                            }
                        }
                        NetMessages::Debug(text) => {
                            if let Some(mut found) =
                                self.handle.find_name::<TextView>(DEBUG_TEXT_ID)
                            {
                                found.set_content(text);
                            } else {
                                error!("Debug view {} could not be found!", DEBUG_TEXT_ID);
                            }
                        }
                    },
                },
                InternalUiMsg::StartAnimate(signal, on_off) => {
                    let sender_clone = tui_sender.clone();
                    match on_off {
                        Status::ON => {
                            self.toggle_alive(signal.clone(), AliveSym::GoOn);
                            let (already_running, toggle): (bool, &mut bool) = match signal {
                                CollectionPathAlive::BusyPath(nr) => {
                                    (self.alive.paths[nr].runs, &mut self.alive.paths[nr].runs)
                                }
                                CollectionPathAlive::HostSearch => {
                                    (self.alive.host.runs, &mut self.alive.host.runs)
                                }
                            };
                            trace!("received path alive");
                            if !already_running {
                                *toggle = true;
                                // if timer not started, start
                                task::sleep(Duration::from_millis(config::tui::ALIVE_REFRESH_MSEC))
                                    .await;
                                sender_clone
                                    .send(InternalUiMsg::StepAndAnimate(signal.clone()))
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
                InternalUiMsg::StepAndAnimate(which) => {
                    let continue_timeout = match which {
                        CollectionPathAlive::BusyPath(nr) => self.alive.paths[nr].runs,
                        CollectionPathAlive::HostSearch => self.alive.host.runs,
                    };
                    if continue_timeout {
                        self.toggle_alive(which.clone(), AliveSym::GoOn);
                        let sender_clone = tui_sender.clone();
                        task::sleep(Duration::from_millis(config::tui::ALIVE_REFRESH_MSEC)).await;
                        sender_clone
                            .send(InternalUiMsg::StepAndAnimate(which))
                            .unwrap_or_else(|_| {
                                // nothing to be done, that timer is just at the end
                                error!("when does this happen???")
                            });
                    }
                }
                InternalUiMsg::PeerSearchFinished(peer, count) => {
                    //
                }
                InternalUiMsg::Terminate => {
                    self.handle.quit();
                }
            }
        }
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
    ///  let output = adbflib::ctrl::tui::Tui::split_intelligently_ralign_vec(&input_vec, boundary);
    ///  ```
    // todo: not public... only due to testing
    pub fn split_intelligently_ralign_vec(vec: &Vec<String>, max_len: usize) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for el in vec {
            out.push(Self::split_intelligently_ralign(el, max_len));
        }
        out
    }

    pub fn split_intelligently_ralign(el: &String, max_len: usize) -> String {
        let real_len = el.chars().count();
        // simple case
        if real_len < max_len {
            format!("{}{}", " ".repeat(max_len - real_len), el.to_string())
        } else if real_len < max_len / 2 {
            el.chars().skip(real_len - max_len).collect::<String>()
        } else {
            let diff = max_len - SEPARATOR.chars().count();
            let real_middle = diff / 2;
            let offset = if real_middle * 2 < diff { 1 } else { 0 };

            let first_part = el.chars().take(real_middle + offset).collect::<String>();
            let last_part = el.chars().skip(real_len - real_middle).collect::<String>();

            format!("{}{}{}", first_part, SEPARATOR, last_part)
        }
    }

    /// Toggle the alive signal by changing the data inside
    /// signal can be for the path search or net search.   
    fn toggle_alive(&mut self, signal: CollectionPathAlive, on: AliveSym) {
        // retrieve view name and the counter value
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
            // change the data in place
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

    /// Builds up the whole cursive tui: views, lists, etc.
    /// Net features result in a slightly different display, that
    /// found host can be listed.
    ///
    /// Blocks until channel counterpart receiver gives an ok.
    ///
    /// # Arguments
    ///
    /// * `title` - The title to be shown on top
    /// * `paths` - the paths as string to be searched
    /// * `with_net` - if the net list box has to be displayed
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
                .title("Peer list"),
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

                let differentiate_path =
                    Self::split_intelligently_ralign_vec(paths, max_table_width);
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
        let layer = Dialog::around(Layer::new(vertical_layout)).title(format!("Peer: {}", title));
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
        let output = Tui::split_intelligently_ralign_vec(&input, boundary);
        println!("|{:?}|", output);
        println!("|{:?}|", expected);
        output.len() == expected.len() && output.iter().zip(expected.iter()).all(|(e, o)| e == o)
    }

    #[test]
    fn split_intelligently_ralign_exists() {
        let boundary = 20;
        let input_vec: Vec<String> = Vec::new();
        let _ = Tui::split_intelligently_ralign_vec(&input_vec, boundary);
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
