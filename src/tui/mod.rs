extern crate cursive;

use self::cursive::Cursive;
use self::cursive::views::{TextView,DummyView,Dialog};
use self::cursive::traits::*;


use std::sync::{Arc, Mutex};                     // safe containment and locking

pub struct Tui {
    handle : Cursive,
}


static RECT : usize = 4;

impl <'tuilife> Tui {
    pub fn new<'a>() -> Tui {
        Tui {
            handle : cursive::Cursive::new()
        }
    }

    pub fn show<'a>(&'tuilife mut self, number: usize) {

        let rows = number / RECT;
        let cols = number % RECT;

        let mut vertical_layout = cursive::views::Layer{cursive::views::LinearLayout::vertical()}; //cursive::views::LinearLayout::vertical();
        for j in 0..rows {
            for i in 0..cols {
                let the_number = j*RECT+i;

                let item = cursive::views::BoxView::with_fixed_size((20,4),
                            TextView::new(format!("Thread {}",the_number)));

                //item.add_child(cursive::views::BoxView::with_fixed_size((20,4),
                            //TextView::new(format!("Thread {}",the_number))
                              //       .with_id(format!("t{}",the_number))));
                vertical_layout.add_layer(cursive::views::LinearLayout::horizontal().child(item));
            }
            vertical_layout.add_child(horizontal_layout);
        }
        self.handle.add_layer(vertical_layout);        

        self.handle.run();        
    }
} // impl Tui