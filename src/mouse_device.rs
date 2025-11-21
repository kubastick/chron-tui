use hidapi::HidDevice;
use log::trace;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DpiConfig {
    pub presets: [u16; 5],
    pub current_index: u8,
    #[allow(unused)]
    pub enabled_flags: [u8; 6],
}

impl TryFrom<[u8; 64]> for DpiConfig {
    type Error = Box<dyn std::error::Error>;

    fn try_from(data: [u8; 64]) -> Result<Self, Self::Error> {
        if data.len() < 37 {
            return Err("Response too short".into());
        }

        if data[1] != 0x06 || data[2] != 0x00 {
            return Err(format!(
                "Invalid response: cmd={:02x} status={:02x}",
                data[1], data[2]
            )
            .into());
        }

        let presets = [
            u16::from_le_bytes([data[6], data[7]]),
            u16::from_le_bytes([data[8], data[9]]),
            u16::from_le_bytes([data[10], data[11]]),
            u16::from_le_bytes([data[12], data[13]]),
            u16::from_le_bytes([data[14], data[15]]),
        ];

        let current_index = data[27];

        let enabled_flags = [data[31], data[32], data[33], data[34], data[35], data[36]];

        Ok(DpiConfig {
            presets,
            current_index,
            enabled_flags,
        })
    }
}
pub struct MouseDevice {
    device: HidDevice,
}

impl MouseDevice {
    pub fn new(device: HidDevice) -> MouseDevice {
        MouseDevice { device }
    }

    pub fn get_dpi_config(&self) -> Result<DpiConfig, Box<dyn std::error::Error>> {
        trace!("Sending get dpi config command");
        let mut cmd = vec![0xB3];
        cmd.extend_from_slice(&[0x06, 0x00]);
        cmd.resize(64, 0);

        // Without sending command 3 times we do not always get response - firmware bug (?)
        for _i in 0..3 {
            self.device.write(&cmd)?;
            thread::sleep(Duration::from_millis(100));
        }

        trace!("Waiting for dpi config response");
        let mut buf = [0u8; 64];
        let n = self.device.read_timeout(&mut buf, 1000)?;

        if n == 0 {
            return Err("No response from device".into());
        }

        trace!("Got dpi config response: {:?}", buf);
        DpiConfig::try_from(buf)
    }
}
