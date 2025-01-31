use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, time::Duration};

use plugin::manager::PluginManager;
use simplelog::info;
use tokio::time::interval;

use crate::config::Config;

mod plugin;

const TICK_RATE: u64 = 20;

pub struct Controller {
    /* State */
    running: Arc<AtomicBool>,

    /* Config */
    config: Config,

    /* Components */
    plugin: PluginManager,
}

impl Controller {
    pub fn init(config: Config) -> Self {
        Self {
            running: Arc::new(AtomicBool::new(true)),
            config,
            plugin: PluginManager::init(),
        }
    }

    pub async fn run(&mut self) {
        // Setup signal handlers
        self.setup_handlers();

        // Main loop
        let mut interval = interval(Duration::from_millis(1000 / TICK_RATE));
        while self.running.load(Ordering::Relaxed) {
            interval.tick().await;
            self.tick().await;
        }

        // Shutdown
        self.shutdown().await;
    }

    async fn tick(&mut self) {

    }

    async fn shutdown(&mut self) {
        info!("Starting shutdown sequence...");
        info!("Shutdown complete. Bye :)");
    }

    fn setup_handlers(&self) {
        let flag = self.running.clone();
        ctrlc::set_handler(move || {
            info!("Received SIGINT, shutting down...");
            flag.store(false, Ordering::Relaxed);
        }).expect("Failed to set Ctrl+C handler");
    }

    /* Configuration */
    pub fn get_config(&self) -> &Config {
        &self.config
    }
}