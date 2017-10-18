extern crate cursive;

use self::cursive::Cursive;
use self::cursive::views::{TextView,Layer, ListView, LinearLayout,Panel};
use self::cursive::traits::*;


pub struct Tui {
    handle : Cursive,
}


static RECT : usize = 4;

impl <'tuilife> Tui {
    pub fn new<'a>(pathes: &Vec<&str>) -> Tui {
        let mut tui = Tui {
            handle : Cursive::new()
        };
        let rows = pathes.len() / RECT + 1;

        let mut vertical_layout = LinearLayout::vertical();
        for j in 0..rows {
            let mut horizontal_layout = LinearLayout::horizontal();

            let cols = if (j+1) * RECT < pathes.len() {
                RECT
            } else {
                pathes.len() % RECT
            };

            for i in 0..cols {
                let my_number = j * RECT + i;  // j >= 1

                let differentiate_path = Tui::split_intelligent(pathes,20);

                let textview = TextView::new(format!("{:?}",differentiate_path[my_number])).
                                    with_id(format!("t{:?}",my_number));
                let mut listview = ListView::new();
                listview.add_child("label",textview);
                let new_panel = Panel::new(listview);
                horizontal_layout.add_child(new_panel);
            }
            vertical_layout.add_child(horizontal_layout);
        }
        let layer = Layer::new(vertical_layout);
        tui.handle.add_layer(layer);
        tui 
    }


    fn split_intelligent<'a>(vec : &'a Vec<&str>, len: usize) -> Vec<&'a str> {
        let mut out : Vec<&str> = Vec::new();
        for el in vec {
            out.push(&el[1..10]);
        }
        out
    }


    pub fn step(&mut self) -> bool {
            if !self.handle.is_running() {
                return false;
            }

// see https://cafbit.com/post/cursive_writing_terminal_applications_in_rust/
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