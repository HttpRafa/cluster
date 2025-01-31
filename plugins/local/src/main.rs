#![no_main]

use driver::{Local, LocalCloudletWrapper};
use exports::cloudlet::driver::bridge::Guest;
use wit_bindgen::generate;

mod driver;
mod log;
mod storage;

generate!({
    world: "driver",
    path: "../../protocol/wit/",
    additional_derives: [PartialEq, Eq],
});

struct Export;

impl Guest for Export {
    type GenericDriver = Local;
    type GenericCloudlet = LocalCloudletWrapper;
}

export!(Export);
