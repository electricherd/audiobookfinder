#![allow(clippy::missing_safety_doc, clippy::not_unsafe_ptr_arg_deref)]

use allo_isolate::Isolate;
use ffi_helpers::null_pointer_check;
use lazy_static::lazy_static;
use std::{ffi::CStr, io, os::raw};
use tokio::runtime::{Builder, Runtime};

lazy_static! {
    static ref RUNTIME: io::Result<Runtime> = Builder::new()
        .threaded_scheduler()
        .enable_all()
        .core_threads(4)
        .thread_name("adbflib")
        .build();
}

macro_rules! error {
    ($result:expr) => {
        error!($result, 0);
    };
    ($result:expr, $error:expr) => {
        match $result {
            Ok(value) => value,
            Err(e) => {
                ffi_helpers::update_last_error(e);
                return $error;
            }
        }
    };
}

macro_rules! cstr {
    ($ptr:expr) => {
        cstr!($ptr, 0);
    };
    ($ptr:expr, $error:expr) => {{
        null_pointer_check!($ptr);
        error!(unsafe { CStr::from_ptr($ptr).to_str() }, $error)
    }};
}

macro_rules! runtime {
    () => {
        match RUNTIME.as_ref() {
            Ok(rt) => rt,
            Err(_) => {
                return 0;
            }
        }
    };
}

#[no_mangle]
pub unsafe extern "C" fn last_error_length() -> i32 {
    ffi_helpers::error_handling::last_error_length()
}

#[no_mangle]
pub unsafe extern "C" fn error_message_utf8(buf: *mut raw::c_char, length: i32) -> i32 {
    ffi_helpers::error_handling::error_message_utf8(buf, length)
}

#[no_mangle]
pub extern "C" fn file_count_good(dart_port: i64, one_path: *const raw::c_char) -> i32 {
    let rt = runtime!();
    let first_entry: &str = cstr!(one_path);
    // todo: is capable of using more than 1 path but for simplicity only one path now
    let paths = vec![first_entry.to_string()];
    let t = Isolate::new(dart_port).task(adbfbinlib::file_count_good(paths));
    rt.spawn(t);
    1
}

#[no_mangle]
pub extern "C" fn find_new_peer(dart_port: i64) -> u64 {
    let rt = runtime!();
    let t = Isolate::new(dart_port).task(adbfbinlib::find_new_peer());
    rt.spawn(t);
    1
}
