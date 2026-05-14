use std::time::{Duration, Instant};
use anyhow::Result;
use serialport::SerialPort;
use std::io::{Read, Write};

pub const MAX_DEVICES: usize = 16;
pub const PING_INTERVAL: Duration = Duration::from_millis(1000);
pub const TIMEOUT_TIME: Duration = Duration::from_millis(20000);
pub const SEARCH_INTERVAL: Duration = Duration::from_millis(5000);
pub const RESYNC_INTERVAL: Duration = Duration::from_millis(5000);

pub const SEARCH_RESPONSE_LENGTH: usize = 26;
pub const PING_RESPONSE_LENGTH: usize = 25;
pub const PART_0_LENGTH: usize = 1015;
pub const PART_1_LENGTH: usize = 911;

pub const SEARCH_RESPONSE: u8 = 0;
pub const PING_RESPONSE: u8 = 4;
pub const DUMP_RESPONSE: u8 = 16;
pub const DIRECT_COMMAND: u8 = 32;

pub const ID_BYTE: usize = 4;
pub const COMMAND_BYTE: usize = 6;
pub const PARAM_COUNT_BYTE: usize = 7;
pub const CHANNEL_BYTE: usize = 8;
pub const PARAM_BYTE: usize = 9;
#[allow(dead_code)]
pub const DUMP_PART_BYTE: usize = 9;
pub const VALUE_HI_BYTE: usize = 10;
pub const VALUE_LOW_BYTE: usize = 11;
pub const PART_BYTE: usize = 12;

pub const COMMAND_START: u8 = 240;
pub const TERMINATOR: u8 = 247;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct HighByte {
    pub part: i32,
    pub byte: i32,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct MiddleBit {
    pub part: i32,
    pub byte: i32,
    pub index: i32,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct LowByte {
    pub part: i32,
    pub byte: i32,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct DataLocation {
    pub low: LowByte,
    pub middle: MiddleBit,
    pub high: HighByte,
}

struct Device {
    last_response: Option<Instant>,
    response: [u8; SEARCH_RESPONSE_LENGTH],
}

impl Default for Device {
    fn default() -> Self {
        Self {
            last_response: None,
            response: [0; SEARCH_RESPONSE_LENGTH],
        }
    }
}

pub struct Ultradrive {
    devices: [Device; MAX_DEVICES],
    dump0: [u8; PART_0_LENGTH],
    dump1: [u8; PART_1_LENGTH],
    ping_response: [u8; PING_RESPONSE_LENGTH],
    last_resync: Option<Instant>,
    last_ping: Option<Instant>,
    last_search: Option<Instant>,
    selected_device: usize,
    invalidate_sync: bool,
    is_first_run: bool,
    #[allow(dead_code)]
    flow_control: bool,
    reading_command: bool,
    serial_read: usize,
    serial_buffer: [u8; PART_0_LENGTH],
    port: Option<Box<dyn SerialPort>>,
    ws_tx: Option<tokio::sync::broadcast::Sender<Vec<u8>>>,
}

impl Ultradrive {
    pub fn new(port: Option<Box<dyn SerialPort>>) -> Self {
        Self {
            devices: Default::default(),
            dump0: [0; PART_0_LENGTH],
            dump1: [0; PART_1_LENGTH],
            ping_response: [0; PING_RESPONSE_LENGTH],
            last_resync: None,
            last_ping: None,
            last_search: None,
            selected_device: 0,
            invalidate_sync: false,
            is_first_run: true,
            flow_control: false,
            reading_command: false,
            serial_read: 0,
            serial_buffer: [0; PART_0_LENGTH],
            port,
            ws_tx: None,
        }
    }

    pub fn set_ws_tx(&mut self, tx: tokio::sync::broadcast::Sender<Vec<u8>>) {
        self.ws_tx = Some(tx);
    }

    pub fn process_incoming(&mut self) -> Result<()> {
        let port = match self.port.as_mut() {
            Some(p) => p,
            None => return Ok(()),
        };

        let mut buf = [0u8; 128];
        match port.read(&mut buf) {
            Ok(n) if n > 0 => {
                if let Some(tx) = &self.ws_tx {
                    let _ = tx.send(buf[..n].to_vec());
                }
                for i in 0..n {
                    self.read_commands(buf[i]);
                }
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(e) => return Err(e.into()),
        }

        let now = Instant::now();

        if self.is_first_run {
            self.is_first_run = false;
            self.last_search = Some(now);
            self.search()?;
            return Ok(());
        }

        if self.last_search.map_or(true, |ls| now.duration_since(ls) >= SEARCH_INTERVAL) {
            log::info!("Searching for devices.");
            self.last_search = Some(now);
            self.search()?;
            return Ok(());
        }

        if self.last_ping.map_or(true, |lp| now.duration_since(lp) >= PING_INTERVAL) {
            self.last_ping = Some(now);
            log::debug!("Pinging selected device.");
            self.ping(self.selected_device as u8)?;
            return Ok(());
        }

        if self.last_resync.map_or(true, |lr| now.duration_since(lr) >= RESYNC_INTERVAL) {
            if let Some(last_resp) = self.devices[self.selected_device].last_response {
                if now.duration_since(last_resp) < TIMEOUT_TIME {
                    self.last_resync = Some(now);
                    log::info!("Syncing selected device.");
                    self.set_transmit_mode(self.selected_device as u8)?;
                    self.dump(self.selected_device as u8, 0)?;
                    self.dump(self.selected_device as u8, 1)?;
                }
            }
        }

        Ok(())
    }

    fn read_commands(&mut self, b: u8) {
        if b == COMMAND_START {
            log::debug!("Started receiving data from device");
            self.reading_command = true;
            self.serial_read = 0;
        }

        if self.reading_command && self.serial_read < PART_0_LENGTH {
            self.serial_buffer[self.serial_read] = b;
            self.serial_read += 1;
        } else {
            self.reading_command = false;
            self.serial_read = 0;
            return;
        }

        if b == TERMINATOR {
            log::debug!("Received end of data from device");
            self.reading_command = false;
            
            // Standard Behringer Header: [0xF0, 0x00, 0x20, 0x32, ID]
            // We check the first 4 bytes.
            let vendor_header = [0xF0, 0x00, 0x20, 0x32];
            if self.serial_read < 5 || self.serial_buffer[..4] != vendor_header {
                return;
            }

            let command = self.serial_buffer[COMMAND_BYTE];
            match command {
                SEARCH_RESPONSE => {
                    if self.serial_read == SEARCH_RESPONSE_LENGTH {
                        log::info!("Received search response");
                        let device_id = self.serial_buffer[ID_BYTE] as usize;
                        if device_id < MAX_DEVICES {
                            self.devices[device_id].last_response = Some(Instant::now());
                            self.devices[device_id].response.copy_from_slice(&self.serial_buffer[..SEARCH_RESPONSE_LENGTH]);
                        }
                    }
                }
                DUMP_RESPONSE => {
                    let part = self.serial_buffer[PART_BYTE];
                    if part == 0 {
                        if !self.invalidate_sync && self.serial_read == PART_0_LENGTH {
                            log::info!("Received state part 0");
                            self.dump0.copy_from_slice(&self.serial_buffer[..PART_0_LENGTH]);
                        }
                    } else if part == 1 {
                        if self.invalidate_sync {
                            self.invalidate_sync = false;
                        } else if self.serial_read == PART_1_LENGTH {
                            log::info!("Received state part 1");
                            self.dump1.copy_from_slice(&self.serial_buffer[..PART_1_LENGTH]);
                        }
                    }
                }
                PING_RESPONSE => {
                    if self.serial_read == PING_RESPONSE_LENGTH {
                        log::info!("Received ping response");
                        self.ping_response.copy_from_slice(&self.serial_buffer[..PING_RESPONSE_LENGTH]);
                    }
                }
                DIRECT_COMMAND => {
                    let count = self.serial_buffer[PARAM_COUNT_BYTE] as usize;
                    log::info!("Received direct commands: {}", count);
                    for i in 0..count {
                        let offset = 4 * i;
                        let channel = self.serial_buffer[CHANNEL_BYTE + offset];
                        let param = self.serial_buffer[PARAM_BYTE + offset];
                        let value_high = self.serial_buffer[VALUE_HI_BYTE + offset];
                        let value_low = self.serial_buffer[VALUE_LOW_BYTE + offset];
                        self.patch_buffer_skeleton(channel, param, value_high, value_low);
                    }
                }
                _ => {}
            }
        }
    }

    fn patch_buffer_skeleton(&mut self, channel: u8, param: u8, high: u8, low: u8) {
        use crate::locations::{SETUP_LOCATIONS, INPUT_LOCATIONS, OUTPUT_LOCATIONS};

        if channel == 0 {
            let index = if param <= 11 { param as i32 - 2 } else { param as i32 - 10 };
            if index >= 0 && index < SETUP_LOCATIONS.len() as i32 {
                self.patch_buffer(low, high, SETUP_LOCATIONS[index as usize]);
            }
        } else if channel >= 1 && channel <= 4 {
            let channel_idx = (channel - 1) as usize;
            let param_idx = (param as i32 - 2) as usize;
            if param_idx < INPUT_LOCATIONS[channel_idx].len() {
                self.patch_buffer(low, high, INPUT_LOCATIONS[channel_idx][param_idx]);
            }
        } else if channel >= 5 && channel <= 10 {
            let channel_idx = (channel - 5) as usize;
            let param_idx = (param as i32 - 2) as usize;
            if param_idx < OUTPUT_LOCATIONS[channel_idx].len() {
                self.patch_buffer(low, high, OUTPUT_LOCATIONS[channel_idx][param_idx]);
            }
        }
    }

    pub fn patch_buffer(&mut self, low: u8, high: u8, l: DataLocation) {
        let dump: &mut [u8] = if l.low.part == 0 { &mut self.dump0 } else { &mut self.dump1 };
        dump[l.low.byte as usize] = low;

        if l.middle.byte >= 0 {
            let dump_m: &mut [u8] = if l.middle.part == 0 { &mut self.dump0 } else { &mut self.dump1 };
            if (high & 1) != 0 {
                dump_m[l.middle.byte as usize] |= 1 << l.middle.index;
            } else {
                dump_m[l.middle.byte as usize] &= !(1 << l.middle.index);
            }
        }

        if l.high.byte >= 0 {
            let dump_h: &mut [u8] = if l.high.part == 0 { &mut self.dump0 } else { &mut self.dump1 };
            dump_h[l.high.byte as usize] = high >> 1;
        }
    }

    pub fn search(&mut self) -> Result<()> {
        if let Some(port) = self.port.as_mut() {
            let cmd = [0xF0, 0x00, 0x20, 0x32, 0x20, 0x0E, 0x40, TERMINATOR];
            port.write_all(&cmd)?;
        }
        Ok(())
    }

    pub fn ping(&mut self, device_id: u8) -> Result<()> {
        if let Some(port) = self.port.as_mut() {
            let cmd = [0xF0, 0x00, 0x20, 0x32, device_id, 0x0E, 0x44, 0x00, 0x00, TERMINATOR];
            port.write_all(&cmd)?;
        }
        Ok(())
    }

    pub fn dump(&mut self, device_id: u8, part: u8) -> Result<()> {
        if let Some(port) = self.port.as_mut() {
            let cmd = [0xF0, 0x00, 0x20, 0x32, device_id, 0x0E, 0x50, 0x01, 0x00, part, TERMINATOR];
            port.write_all(&cmd)?;
        }
        Ok(())
    }

    pub fn set_transmit_mode(&mut self, device_id: u8) -> Result<()> {
        if let Some(port) = self.port.as_mut() {
            let cmd = [0xF0, 0x00, 0x20, 0x32, device_id, 0x0E, 0x3F, 0x0C, 0x00, TERMINATOR];
            port.write_all(&cmd)?;
        }
        Ok(())
    }

    pub fn write_device(&self, buf: &mut Vec<u8>) {
        // ALWAYS write full buffers to avoid UI parsing crashes
        buf.extend_from_slice(&self.dump0);
        buf.extend_from_slice(&self.dump1);
    }

    pub fn write_device_status(&self, buf: &mut Vec<u8>) {
        // ALWAYS write full buffer to avoid UI parsing crashes
        buf.extend_from_slice(&self.ping_response);
    }

    pub fn write_devices(&self, buf: &mut Vec<u8>) {
        let now = Instant::now();
        for device in &self.devices {
            if let Some(last_resp) = device.last_response {
                if now.duration_since(last_resp) < TIMEOUT_TIME {
                    buf.extend_from_slice(&device.response);
                }
            }
        }
    }

    pub fn set_selected(&mut self, device_id: usize) {
        if device_id < MAX_DEVICES {
            self.selected_device = device_id;
        }
    }

    pub fn get_selected(&self) -> usize {
        self.selected_device
    }

    pub fn process_outgoing(&mut self, data: &[u8]) -> Result<()> {
        // Standard Behringer Header: [0xF0, 0x00, 0x20, 0x32, ID]
        // We only check the first 4 bytes to allow any device ID.
        let vendor_header = [0xF0, 0x00, 0x20, 0x32];
        if data.len() < 5 || data[..4] != vendor_header {
            return Ok(());
        }

        let command = data[COMMAND_BYTE];
        if command == DIRECT_COMMAND {
            self.invalidate_sync = true;
            let count = data[PARAM_COUNT_BYTE] as usize;
            for i in 0..count {
                let offset = 4 * i;
                let channel = data[CHANNEL_BYTE + offset];
                let param = data[PARAM_BYTE + offset];
                let value_high = data[VALUE_HI_BYTE + offset];
                let value_low = data[VALUE_LOW_BYTE + offset];
                self.patch_buffer_skeleton(channel, param, value_high, value_low);
            }
        }
        
        if let Some(port) = self.port.as_mut() {
            port.write_all(data)?;
        }
        
        Ok(())
    }

    pub fn set_port(&mut self, port: Box<dyn SerialPort>) {
        self.port = Some(port);
        self.is_first_run = true;
        self.last_search = None;
        self.last_ping = None;
        self.last_resync = None;
        self.devices = Default::default();
        // Clear buffers on new port
        self.dump0 = [0; PART_0_LENGTH];
        self.dump1 = [0; PART_1_LENGTH];
        self.ping_response = [0; PING_RESPONSE_LENGTH];
        log::info!("Serial port updated.");
    }

    pub fn close_port(&mut self) {
        self.port = None;
        log::info!("Serial port closed.");
    }

    pub fn get_port_active(&self) -> Option<&Box<dyn SerialPort>> {
        self.port.as_ref()
    }
}
