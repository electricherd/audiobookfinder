extern crate cursive;

use self::cursive::{Cursive};
use self::cursive::views::{TextView,Layer, ListView, LinearLayout,Panel};
use self::cursive::traits::*;

use std::iter::Iterator;
use mpsc;

use ctrl::{SystemMsg,UiMsg,ReceiveDialog};

pub struct Tui {
    handle : Cursive,
    pub ui_receiver: mpsc::Receiver<UiMsg>,
    pub ui_sender: mpsc::Sender<UiMsg>,
    pub system_sender: mpsc::Sender<SystemMsg>,
} 


static RECT : usize = 40;
static SEPERATOR : &str = "..";
static DEBUG_TEXT_ID : &str = "debug_info";
static PATHS_PREFIX_ID : &str = "pf";

impl <'tuilife> Tui {
    pub fn new<'a>(system: mpsc::Sender<SystemMsg>, pathes: &Vec<String>) -> Tui {

        let (_ui_sender, _ui_receiver) = mpsc::channel::<UiMsg>();
        let mut tui = Tui {
            handle : Cursive::new(),
            ui_sender : _ui_sender,
            ui_receiver : _ui_receiver,
            system_sender : system
        };
        let screen_size = tui.handle.screen_size();
        let max_cols = screen_size.x / RECT;


        let rows = pathes.len() / max_cols + 1;
        // 80% of rect size
        let max_table_width = RECT * 5 / 6;


        let mut vertical_layout = LinearLayout::vertical();
        for j in 0..rows {
            let mut horizontal_layout = LinearLayout::horizontal();

            let cols = if j < rows-1 { max_cols }else{ pathes.len() % max_cols };

            for i in 0..cols {
                let my_number = j * max_cols + i;  // j >= 1

                let differentiate_path = Tui::split_intelligent(pathes,max_table_width);

                let textview = TextView::new(format!("{}",differentiate_path[my_number]))
                                    .with_id(format!("{}{:?}",PATHS_PREFIX_ID,my_number)); // from trait
                           
                let mut listview = ListView::new();
                listview.add_child(": ",textview);
                let new_panel = Panel::new(listview);
                horizontal_layout.add_child(new_panel);
            }
            vertical_layout.add_child(horizontal_layout);
        }

        // debug output here
        let debug_text = max_table_width;
        vertical_layout.add_child(Panel::new(
                          TextView::new(
                           format!("debug_text: {}",debug_text))
                    .with_id(DEBUG_TEXT_ID)));

        let layer = Layer::new(vertical_layout);
        tui.handle.add_layer(layer);

        // test this, to update every with 20fps / this should be done when something changes ..... grrrr
        tui.handle.set_fps(20);
        tui 
    }


    fn split_intelligent<'a>(vec : &Vec<String>, max_len: usize) -> Vec<String> {
        let mut out : Vec<String> = Vec::new();
        for el in vec {
            let real_len = el.chars().count();
            // simple case
            if real_len < max_len {
                out.push(format!("{}{}", " ".repeat(max_len - real_len),el.to_string()));
            } else if real_len < max_len/2 {
                out.push(el.chars().skip(real_len-max_len).collect::<String>());
            } else {
                let diff = max_len-SEPERATOR.chars().count();
                let real_middle = diff / 2;
                let offset = if real_middle*2 < diff { 1 } else { 0 };

                let first_part = el.chars().take(real_middle+offset).collect::<String>();
                let last_part = el.chars().skip(real_len-real_middle).collect::<String>();

                out.push(format!("{}{}{}",first_part 
                                        ,SEPERATOR
                                        ,last_part));
            }
        }
        out
    }


    pub fn step(&mut self) -> bool {
        if !self.handle.is_running() {
            return false;
        }

       while let Some(message) = self.ui_receiver.try_iter().next() {
           match message {
               UiMsg::Update(recv_dialog,text) => {
                match recv_dialog {
                    ReceiveDialog::PathNr{nr} => {
                       let mut output = self.handle
                           .find_id::<TextView>(&format!("{}{}",PATHS_PREFIX_ID,nr))
                           .unwrap();
                       output.set_content(text);
                    },
                    ReceiveDialog::Debug => {
                       let mut output = self.handle
                           .find_id::<TextView>(DEBUG_TEXT_ID)
                           .unwrap();
                       output.set_content(text);
                    }
                }
               }
           }
       }

        // step through the TUI
        self.handle.step();
        true
    }
} // impl Tui