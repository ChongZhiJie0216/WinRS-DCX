use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use std::time::Duration;
use anyhow::Result;

pub enum ConnectionCommand {
    Connect { port: String, baud_rate: u32 },
    Disconnect,
}

pub struct ConnectionManager {
    command_tx: mpsc::Sender<ConnectionCommand>,
}

impl ConnectionManager {
    pub fn new(
        device_manager: Arc<Mutex<crate::ultradrive::Ultradrive>>,
        _timeout_duration: Duration,
    ) -> Self {
        let (tx, mut rx) = mpsc::channel::<ConnectionCommand>(10);
        let current_port: Arc<Mutex<Option<(String, u32)>>> = Arc::new(Mutex::new(None));

        // Background Task
        let cp = current_port.clone();
        tokio::spawn(async move {
            let mut attempt_count = 0;
            let max_attempts = 10;

            loop {
                let mut check_now = false;

                tokio::select! {
                    Some(cmd) = rx.recv() => {
                        match cmd {
                            ConnectionCommand::Connect { port, baud_rate } => {
                                log::info!("Received Connect command for {} at {}", port, baud_rate);
                                let mut cp = cp.lock().await;
                                *cp = Some((port, baud_rate));
                                attempt_count = 0;
                                check_now = true;
                            }
                            ConnectionCommand::Disconnect => {
                                log::info!("Received Disconnect command");
                                let mut cp = cp.lock().await;
                                *cp = None;
                                let mut dm = device_manager.lock().await;
                                dm.close_port();
                                attempt_count = 0;
                            }
                        }
                    }
                    _ = tokio::time::sleep(Duration::from_secs(2)) => {
                        check_now = true;
                    }
                }

                if check_now {
                    let cp_lock = cp.lock().await;
                    if let Some((port_name, baud)) = &*cp_lock {
                        if port_name.trim().is_empty() {
                            continue;
                        }
                        
                        let mut dm = device_manager.lock().await;
                        if dm.get_port_active().is_none() {
                            if attempt_count >= max_attempts {
                                log::warn!("Connection timeout reached for {}. Stopping auto-reconnect.", port_name);
                                continue;
                            }

                            attempt_count += 1;
                            log::info!("Attempting to connect to {} (attempt {}/{})", port_name, attempt_count, max_attempts);
                            
                            // If connecting at 38400, "prime" the port at 9600 first to wake up the device.
                            // This resolves issues where the device won't sync at 38400 on cold start.
                            if *baud == 38400 {
                                log::info!("Priming port {} at 9600 baud...", port_name);
                                let prime_res = serialport::new(port_name, 9600)
                                    .timeout(Duration::from_millis(100))
                                    .open();
                                match prime_res {
                                    Ok(_) => {
                                        log::info!("Priming successful, waiting for port to reset...");
                                        tokio::time::sleep(Duration::from_millis(500)).await;
                                    }
                                    Err(e) => {
                                        log::warn!("Priming failed for {}: {}", port_name, e);
                                    }
                                }
                            }

                            match serialport::new(port_name, *baud)
                                .timeout(Duration::from_millis(100))
                                .open() {
                                    Ok(mut port) => {
                                        log::info!("Successfully connected to {}", port_name);

                                        // Clear buffers and toggle DTR/RTS to ensure a clean state
                                        let _ = port.write_data_terminal_ready(true);
                                        let _ = port.write_request_to_send(true);
                                        let _ = port.clear(serialport::ClearBuffer::All);

                                        dm.set_port(port);
                                        attempt_count = 0;
                                    }
                                    Err(e) => {
                                        log::error!("Connection failed for {}: {}", port_name, e);
                                    }
                                }
                        }
                    }
                }
            }
        });

        Self { command_tx: tx }
    }

    pub async fn send_command(&self, cmd: ConnectionCommand) -> Result<()> {
        self.command_tx.send(cmd).await.map_err(|e| anyhow::anyhow!(e))
    }
}
