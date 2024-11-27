#![no_std]
#![no_main]

use core::cell::RefCell;

use alloc::rc::Rc;
use noli::{entry_point, println};
use saba_core::browser::Browser;
use ui_wasabi::app::WasabiUI;

extern crate alloc;

fn main() -> u64 {
    let browser = Browser::new();

    let ui = Rc::new(RefCell::new(WasabiUI::new(browser)));

    match ui.borrow_mut().start() {
        Ok(_) => {}
        Err(e) => {
            println!("browser fails to start {:?}", e);
            return 1;
        }
    }
    0
}

entry_point!(main);
