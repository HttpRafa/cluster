use std::{
    collections::HashMap,
    io::{BufRead, BufReader, BufWriter, Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

use simplelog::debug;

use crate::storage::Storage;

use super::{
    generated::cloudlet::driver::{
        self,
        process::{Directory, KeyValue, StdReader},
        types::{ErrorMessage, Reference},
    },
    DriverProcess, WasmDriverState,
};

impl driver::process::Host for WasmDriverState {
    fn spawn_process(
        &mut self,
        command: String,
        args: Vec<String>,
        environment: Vec<KeyValue>,
        directory: Directory,
    ) -> Result<u32, String> {
        let driver = self.handle.upgrade().ok_or("Failed to upgrade handle")?;
        let process_dir = self.get_directory(&driver.name, &directory)?;
        let environment: HashMap<_, _> = environment
            .into_iter()
            .map(|kv| (kv.key, kv.value))
            .collect();

        debug!("Spawning process: {} {:?}", command, args);
        let mut command = Command::new(command);
        command
            .args(args)
            .current_dir(process_dir)
            .envs(environment)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut process = command
            .spawn()
            .map_err(|e| format!("Failed to spawn process: {}", e))?;
        let pid = process.id();

        let stdout = BufReader::new(
            process
                .stdout
                .take()
                .ok_or("Failed to open stdout of process")?,
        );
        let stderr = BufReader::new(
            process
                .stderr
                .take()
                .ok_or("Failed to open stderr of process")?,
        );
        let stdin = BufWriter::new(
            process
                .stdin
                .take()
                .ok_or("Failed to open stdin of process")?,
        );

        driver
            .data
            .processes
            .write()
            .map_err(|_| "Failed to acquire write lock on processes")?
            .insert(
                pid,
                DriverProcess {
                    process,
                    stdin,
                    stdout,
                    stderr,
                },
            );

        Ok(pid)
    }

    fn kill_process(&mut self, pid: u32) -> Result<(), ErrorMessage> {
        let driver = self.handle.upgrade().ok_or("Failed to upgrade handle")?;
        let mut processes = driver
            .data
            .processes
            .write()
            .map_err(|_| "Failed to acquire write lock on processes")?;

        debug!("Killing process: {}", pid);
        if let Some(mut process) = processes.remove(&pid) {
            process
                .process
                .kill()
                .map_err(|e| format!("Failed to kill process: {}", e))
        } else {
            Ok(())
        }
    }

    fn drop_process(&mut self, pid: u32) -> Result<bool, ErrorMessage> {
        let driver = self.handle.upgrade().ok_or("Failed to upgrade handle")?;
        let mut processes = driver
            .data
            .processes
            .write()
            .map_err(|_| "Failed to acquire write lock on processes")?;

        debug!("Dropping process: {}", pid);
        Ok(processes.remove(&pid).is_some())
    }

    fn try_wait(&mut self, pid: u32) -> Result<Option<i32>, ErrorMessage> {
        let driver = self.handle.upgrade().ok_or("Failed to upgrade handle")?;
        let mut processes = driver
            .data
            .processes
            .write()
            .map_err(|_| "Failed to acquire write lock on processes")?;

        if let Some(process) = processes.get_mut(&pid) {
            process
                .process
                .try_wait()
                .map_err(|e| format!("Failed to wait for process: {}", e))
                .map(|status| status.and_then(|s| s.code()))
        } else {
            Ok(None)
        }
    }

    fn read_line(&mut self, pid: u32, std: StdReader) -> Result<(u32, String), ErrorMessage> {
        let driver = self.handle.upgrade().ok_or("Failed to upgrade handle")?;
        let mut processes = driver
            .data
            .processes
            .write()
            .map_err(|_| "Failed to acquire write lock on processes")?;

        if let Some(process) = processes.get_mut(&pid) {
            let mut buffer = String::new();
            let bytes = match std {
                StdReader::Stdout => process.stdout.read_line(&mut buffer),
                StdReader::Stderr => process.stderr.read_line(&mut buffer),
            }
            .map_err(|error| format!("Failed to read from process: {}", error))?;
            Ok((bytes as u32, buffer))
        } else {
            Err("Process does not exist".to_string())
        }
    }

    fn has_data_left(&mut self, pid: u32, std: StdReader) -> Result<bool, ErrorMessage> {
        let driver = self.handle.upgrade().ok_or("Failed to upgrade handle")?;
        let mut processes = driver
            .data
            .processes
            .write()
            .map_err(|_| "Failed to acquire write lock on processes")?;

        if let Some(process) = processes.get_mut(&pid) {
            let has_data = match std {
                StdReader::Stdout => process.stdout.has_data_left(),
                StdReader::Stderr => process.stderr.has_data_left(),
            }
            .map_err(|error| format!("Failed to check buffer of process: {}", error))?;
            Ok(has_data)
        } else {
            Err("Process does not exist".to_string())
        }
    }

    fn read(
        &mut self,
        pid: u32,
        buf_size: u32,
        std: StdReader,
    ) -> Result<(u32, Vec<u8>), ErrorMessage> {
        let driver = self.handle.upgrade().ok_or("Failed to upgrade handle")?;
        let mut processes = driver
            .data
            .processes
            .write()
            .map_err(|_| "Failed to acquire write lock on processes")?;

        if let Some(process) = processes.get_mut(&pid) {
            let mut buffer = Vec::with_capacity(buf_size as usize);
            let bytes = match std {
                StdReader::Stdout => process.stdout.read(&mut buffer),
                StdReader::Stderr => process.stderr.read(&mut buffer),
            }
            .map_err(|e| format!("Failed to read from process: {}", e))?;
            Ok((bytes as u32, buffer))
        } else {
            Err("Process does not exist".to_string())
        }
    }

    fn read_to_end(&mut self, pid: u32, std: StdReader) -> Result<(u32, Vec<u8>), ErrorMessage> {
        let driver = self.handle.upgrade().ok_or("Failed to upgrade handle")?;
        let mut processes = driver
            .data
            .processes
            .write()
            .map_err(|_| "Failed to acquire write lock on processes")?;

        if let Some(process) = processes.get_mut(&pid) {
            let mut buffer = Vec::new();
            let bytes = match std {
                StdReader::Stdout => process.stdout.read_to_end(&mut buffer),
                StdReader::Stderr => process.stderr.read_to_end(&mut buffer),
            }
            .map_err(|e| format!("Failed to read from process: {}", e))?;
            Ok((bytes as u32, buffer))
        } else {
            Err("Process does not exist".to_string())
        }
    }

    fn write_stdin(&mut self, pid: u32, data: Vec<u8>) -> Result<(), ErrorMessage> {
        let driver = self.handle.upgrade().ok_or("Failed to upgrade handle")?;
        let mut processes = driver
            .data
            .processes
            .write()
            .map_err(|_| "Failed to acquire write lock on processes")?;

        if let Some(process) = processes.get_mut(&pid) {
            process
                .stdin
                .write_all(&data)
                .map_err(|e| format!("Failed to write to stdin of process: {}", e))?;
            Ok(())
        } else {
            Err("Process does not exist".to_string())
        }
    }
}

impl WasmDriverState {
    pub fn get_directory(
        &self,
        driver_name: &str,
        directory: &Directory,
    ) -> Result<PathBuf, String> {
        match &directory.reference {
            Reference::Controller => Ok(PathBuf::from(".").join(&directory.path)),
            Reference::Data => {
                Ok(Storage::get_data_folder_for_driver(driver_name).join(&directory.path))
            }
            Reference::Configs => {
                Ok(Storage::get_config_folder_for_driver(driver_name).join(&directory.path))
            }
        }
    }
}
