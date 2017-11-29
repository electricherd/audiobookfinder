extern crate cursive;

use self::cursive::{Cursive};
use self::cursive::align;
use self::cursive::views::{Dialog,TextView,Layer, ListView, LinearLayout,Panel};
use self::cursive::traits::*; //{Identifiable,select};

use std::iter::Iterator;
use mpsc;

use ctrl::{SystemMsg,UiMsg,ReceiveDialog,Alive};

pub struct Tui {
    handle : Cursive,
    pub ui_receiver: mpsc::Receiver<UiMsg>,
    pub ui_sender: mpsc::Sender<UiMsg>,
    pub system_sender: mpsc::Sender<SystemMsg>,

    alive : AliveDisplayData
} 

struct AliveDisplayData {
    host   : usize,
    pathes : Vec<usize>
}



static RECT : usize = 40;
static SEPERATOR : &str = "..";
static STR_START_ALIVE : &str = "*";

static STR_ALIVE : [char;4] = ['|','/','-','\\'];


static DEBUG_TEXT_ID : &str = "debug_info";
static VIEW_LIST_HOST : &str = "hostlist";

static PATHS_PREFIX_ID : &str = "pf";
static ID_HOST_INDEX : &str = "id_host";
static ID_HOST_NUMBER : &str = "id_max";
static ID_HOST_ALIVE : &str = "id_host_alive";



impl Tui {
    pub fn new(title: String, system: mpsc::Sender<SystemMsg>, pathes: &Vec<String>, with_net: bool) -> Tui {

        let later_handle = Cursive::new();

        let screen_size = later_handle.screen_size();
        let max_cols = screen_size.x / RECT;


        let rows = pathes.len() / max_cols + 1;
        // 80% of rect size
        let max_table_width = RECT * 5 / 6;


        let mut vertical_layout = LinearLayout::vertical();

        // add host list on top
        if with_net {                    
            vertical_layout.add_child(
                Dialog::around(                
                    LinearLayout::vertical()
                    .child(
                        ListView::new().with_id(VIEW_LIST_HOST).fixed_size((20,5)))
                    .child(LinearLayout::horizontal()
                        .child(TextView::new(format!("{}",STR_START_ALIVE))
                            .with_id(ID_HOST_ALIVE))
                        .child(TextView::new(format!(" Looking at ")))
                        .child(TextView::new(format!("{}",0))
                            .h_align(align::HAlign::Right)
                            .with_id(ID_HOST_INDEX)
                        )
                        .child(TextView::new(format!("/")))
                        .child(TextView::new(format!("{}",0))
                            .h_align(align::HAlign::Left)
                            .with_id(ID_HOST_NUMBER)
                        )
                    )
                ).title("Host list")
            );
        }

        // draw all pathes
        for j in 0..rows {
            let mut horizontal_layout = LinearLayout::horizontal();

            let cols = if j < rows-1 { max_cols }else{ pathes.len() % max_cols };

            for i in 0..cols {
                let my_number = j * max_cols + i;  // j >= 1

                let differentiate_path = Tui::split_intelligent(pathes,max_table_width);
                let path_name = format!("{}{}",PATHS_PREFIX_ID,my_number);

                horizontal_layout.add_child(
                     Panel::new(LinearLayout::horizontal()
                        .child(
                            // and link with an id
                            TextView::new(format!("{}",STR_START_ALIVE))
                            .with_id(path_name)
                        )
                        .child(
                            TextView::new(format!("{}",differentiate_path[my_number]))
                        )
                    ));
            }
            vertical_layout.add_child(horizontal_layout);
        }

        // debug output here
        let debug_text = max_table_width;
        vertical_layout.add_child(Panel::new(
                          TextView::new(
                           format!("debug_text: {}",debug_text))
                    .with_id(DEBUG_TEXT_ID)));

        let layer = Dialog::around(
                        Layer::new(vertical_layout)
                        ).title(format!("server uuid: {}",title));


        // now build the actual TUI object
        let (_ui_sender, _ui_receiver) = mpsc::channel::<UiMsg>();
        let mut tui = Tui {
            handle : later_handle,
            ui_sender : _ui_sender,
            ui_receiver : _ui_receiver,
            system_sender : system,

            alive : AliveDisplayData { host: 0, pathes: vec![0;pathes.len()]}
        };
        tui.handle.add_layer(layer);

        // test this, to update every with 20fps / this should be done when something changes ..... grrrr
        tui.handle.set_fps(20);

        // quit by 'q' key
        tui.handle.add_global_callback('q', |s| s.quit());
        tui 
    }


    fn split_intelligent(vec : &Vec<String>, max_len: usize) -> Vec<String> {
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

    fn show_alive(&mut self, signal: Alive) {
        let (view_name, counter) = match signal {
            Alive::HOSTSEARCH => { 
                 (ID_HOST_ALIVE.to_string(),&mut self.alive.host)
             }
            Alive::BUSYPATH{nr} => {
                 (format!("{}{}",PATHS_PREFIX_ID,nr),&mut self.alive.pathes[nr])
             }
        };
        let mut output = self.handle
                            .find_id::<TextView>(&view_name);
        if let Some(ref mut found) = output {
             *counter += 1;
             let out = STR_ALIVE[(*counter)%4];
             found.set_content(out.to_string());
        }
    }


    pub fn step(&mut self) -> bool {
        if !self.handle.is_running() {
            return false;
        }

       while let Some(message) = self.ui_receiver.try_iter().next() {
           match message {
               UiMsg::Update(recv_dialog,text) => {
                match recv_dialog {
                    ReceiveDialog::ShowNewPath{nr} => {
                       let mut output = self.handle
                           .find_id::<TextView>(&format!("{}{}",PATHS_PREFIX_ID,nr))
                           .unwrap();
                       output.set_content(text.clone());
                    },
                    ReceiveDialog::ShowNewHost => {
                       let mut host_list = self.handle.find_id::<ListView>(VIEW_LIST_HOST).unwrap();
                       let new_host_entry = TextView::new(format!("{}",text));
                       host_list.add_child("",new_host_entry);
                    }
                    ReceiveDialog::ShowRunning{what} => {
                       self.show_alive(what);
                    },
                    ReceiveDialog::NetStats => {
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