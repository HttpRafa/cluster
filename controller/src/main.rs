use std::time::Instant;

use args::Args;
use clap::Parser;
use colored::Colorize;
use common::init::CloudInit;
use log::info;

use crate::application::Controller;
use crate::config::Config;

mod application;
mod args;
mod config;
mod network;
mod storage;

// Include the build information generated by build.rs
include!(concat!(env!("OUT_DIR"), "/build_info.rs"));

pub const AUTHORS: [&str; 1] = ["HttpRafa"];

fn main() {
    let args = Args::parse();
    CloudInit::init_logging(args.debug);
    CloudInit::print_ascii_art("Atomic Cloud", &VERSION, &AUTHORS);

    let start_time = Instant::now();
    info!(
        "Starting cloud version {}...",
        format!("v{}", VERSION).blue()
    );
    info!("Loading configuration...");

    let configuration = Config::new_filled();
    let controller = Controller::new(configuration);
    info!("Loaded cloud in {:.2?}", start_time.elapsed());
    controller.start();
}
