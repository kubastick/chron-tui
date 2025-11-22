use hidapi::HidDevice;
use log::trace;

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

        let current_index = data[4];

        let enabled_flags = [data[31], data[32], data[33], data[34], data[35], data[36]];

        Ok(DpiConfig {
            presets,
            current_index,
            enabled_flags,
        })
    }
}

#[derive(Debug)]
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

        self.device.write(&cmd)?;

        trace!("Waiting for dpi config response");
        let mut buf = [0u8; 64];
        let n = self.device.read_timeout(&mut buf, 2000)?;

        if n == 0 {
            return Err("No response from device".into());
        }

        trace!("Got dpi config response: {:?}", buf);
        DpiConfig::try_from(buf)
    }

    pub fn get_device_info(&self) -> Result<(), Box<dyn std::error::Error>> {
        trace!("Sending get device info command");

        let mut cmd = vec![0xB5, 0x02, 0x00];
        cmd.resize(21, 0);
        self.device.write(&cmd)?;

        let mut buf = [0u8; 21];
        self.device.read_timeout(&mut buf, 1000)?;
        trace!("Received device info: {:?}", buf);

        Ok(())
    }

    pub fn set_active_dpi_preset(
        &self,
        active_preset: u8,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.device.set_blocking_mode(false)?;
        let mut buf = [0u8; 64];
        while self.device.read(&mut buf).unwrap_or(0) > 0 {
            trace!("Drained old response: {:02x?}", buf);
        }

        if active_preset > 4 {
            return Err("Active preset must be 0-4".into());
        }
        const REPORT_ID: u8 = 0xB5;

        // TODO: Hardcoded DPI preset values
        const STATIC_CMD_PART: [u8; 11] = [
            0x90, 0x01, 0xb0, 0x04, 0x78, 0x05, 0xd0, 0x07, 0x88, 0x13, 0x05,
        ];
        const ACCEPTED_RESPONSE_PREFIX: [u8; 1] = [0xB6];

        trace!("Sending set active dpi preset command");

        let mut cmd = vec![REPORT_ID, 0x40];
        cmd.extend_from_slice(&[active_preset, active_preset, active_preset]);
        cmd.extend_from_slice(&STATIC_CMD_PART);
        cmd.resize(21, 0);
        trace!("Set DPI preset CMD: {cmd:?}");

        let bytes_written = self.device.write(&cmd)?;
        trace!("Wrote {bytes_written} bytes");

        let mut buf = [0u8; 20];

        let n = self.device.read_timeout(&mut buf, 1000)?;
        if n == 0 {
            return Err("No response from device".into());
        }

        if buf.starts_with(&ACCEPTED_RESPONSE_PREFIX) {
            trace!("Device accepted active DPI preset command");
            Ok(())
        } else {
            trace!("Device didn't accepted active DPI preset command: {buf:?}");
            Err("Device returned incorrect response to set active dpi preset command".into())
        }
    }
}
