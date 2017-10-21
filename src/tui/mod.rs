extern crate cursive;

use self::cursive::Cursive;
use self::cursive::views::{TextView,Layer, ListView, LinearLayout,Panel};
use self::cursive::traits::*;

use std::sync::mpsc;
use std::iter::Iterator;


pub enum UiMsg {
    Update(String)
}
pub enum SystemMsg {
    Update(String)
}


pub struct Tui {
    handle : Cursive,
    ui_receiver: mpsc::Receiver<UiMsg>,
    ui_sender: mpsc::Sender<UiMsg>,
    system_sender: mpsc::Sender<SystemMsg>,
}


static RECT : usize = 4;
static SEPERATOR : &str = "...";

impl <'tuilife> Tui {
    pub fn new<'a>(system: mpsc::Sender<SystemMsg>, pathes: &Vec<&str>) -> Tui {

        let (_ui_sender, _ui_receiver) = mpsc::channel::<UiMsg>();
        let mut tui = Tui {
            handle : Cursive::new(),
            ui_sender : _ui_sender,
            ui_receiver : _ui_receiver,
            system_sender : system
        };
        let rows = pathes.len() / RECT + 1;

        let max_table_width = 22;

        let mut vertical_layout = LinearLayout::vertical();
        for j in 0..rows {
            let mut horizontal_layout = LinearLayout::horizontal();

            let cols = if j < rows-1 {
                RECT
            } else {
                pathes.len() % RECT
            };

            for i in 0..cols {
                let my_number = j * RECT + i;  // j >= 1

                let differentiate_path = Tui::split_intelligent(pathes,max_table_width);

                let textview = TextView::new(format!("{}",differentiate_path[my_number])).
                                    with_id(format!("t{:?}",my_number)); // from trait
//                                    .required_size((max_table_width,20));                                    
                let mut listview = ListView::new();
                listview.add_child(": ",textview);
                let new_panel = Panel::new(listview);
                horizontal_layout.add_child(new_panel);
            }
            vertical_layout.add_child(horizontal_layout);
        }

        // debug output here
        let debug_text = tui.handle.screen().layer_sizes().iter().count();
        vertical_layout.add_child(Panel::new(
                          TextView::new(
                           format!("debug_text: {}",debug_text))
                    .with_id("debug_info")));

        let layer = Layer::new(vertical_layout);
        tui.handle.add_layer(layer);
        tui 
    }


    fn split_intelligent<'a>(vec : &'a Vec<&str>, max_len: usize) -> Vec<String> {
        let mut out : Vec<String> = Vec::new();
        for el in vec {
            let real_len = el.chars().count();
            // simple case
            if real_len < max_len {
                out.push(el.to_string());
            } else if real_len < max_len/2 {
                // todo:
                out.push(el.chars().skip(real_len-max_len).collect::<String>());
            } else {
                // todo: why 3??? 2 breaks it
                let real_middle = (real_len-SEPERATOR.chars().count()) / 2; 
                let first_part = el.chars().take(real_middle).collect::<String>();
                let last_part = el.chars().skip(real_len-real_middle+1).collect::<String>();

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

// use https://cafbit.com/post/cursive_writing_terminal_applications_in_rust/
//            // Process any pending UI messages
//            while let Some(message) = self.ui_rx.try_iter().next() {
//                match message {
//                    UiMessage::UpdateOutput(text) => {
//                        let mut output = self.cursive
//                            .find_id::<TextView>("output")
//                            .unwrap();
//                        output.set_content(text);
//                    }
//                }
//            }

            // Step the UI
            self.handle.step();
            true
    }

    pub fn show<'a>(&'tuilife mut self) {
        self.handle.run();        
    }
} // impl Tui