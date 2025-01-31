#![feature(buf_read_has_data_left)]

use application::Controller;
use clap::{ArgAction, Parser};
use common::init::CloudInit;
use config::Config;
use simplelog::info;
use storage::Storage;
use tokio::time::Instant;

mod storage;
mod config;
mod application;

// Include the build information generated by build.rs
include!(concat!(env!("OUT_DIR"), "/build_info.rs"));

pub const AUTHORS: [&str; 1] = ["HttpRafa"];

#[tokio::main]
async fn main() {
    let arguments = Arguments::parse();
    CloudInit::init_logging(arguments.debug, false, Storage::get_latest_log_file());
    CloudInit::print_ascii_art("Atomic Cloud", &VERSION, &AUTHORS);

    let beginning = Instant::now();
    info!("Starting cloud version v{}...", VERSION);
    info!("Initializing controller...");

    let mut controller = Controller::init(Config::parse());
    info!("Loaded cloud in {:.2?}", beginning.elapsed());
    controller.run().await;
}

#[derive(Parser)]
pub struct Arguments {
    #[clap(short, long, help = "Enable debug mode", action = ArgAction::SetTrue)]
    pub debug: bool,
}