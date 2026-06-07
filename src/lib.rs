use hidapi::{HidApi, HidDevice, HidError};
use thistermination::TerminationFull;
// SteelSeries vendorID
const VENDOR_IDS: [u16; 1] = [0x1038];
// ProductIDs for:
//  SteelSeries Aerox 5 Wireless
//  SteelSeries Aerox 5 Wireless Destiny 2 Edition
//  SteelSeries Aerox 5 Wireless Diablo IV Edition
// in wired and wireless mode
const PRODUCT_IDS: [u16; 6] = [0x1854, 0x185E, 0x1862, 0x1852, 0x185C, 0x1860];
const INTERFACE_NUMBER: i32 = 3;
const BATTERY_LEVEL_INDEX: usize = 1;
const BATTERY_LEVEL_PREAMBLE: [u8; 1] = [0xD2];
const MOUSE_OFF: [u8; 2] = [0x40, 0xFF];
const RESPONSE_LENGTH: usize = 2;
const BATTERY_PACKET: [u8; 2] = {
    let mut packet = [0; 2];
    (packet[0], packet[1]) = (0x00, 0xD2);
    packet
};
fn get_battery_state(byte: u8) -> (u8, bool) {
    let charging_flag: u8 = 0b10000000;
    let charging = byte & charging_flag != 0;
    let raw = byte & !charging_flag;
    let level = if raw > 0 { (raw - 1) * 5 } else { 0 };
    (level, charging)
}
#[derive(TerminationFull)]
pub enum DeviceError {
    #[termination(exit_code(1), msg("{0}"))]
    HidError(#[from] HidError),
    #[termination(exit_code(2), msg("No device found."))]
    NoDeviceFound(),
    #[termination(exit_code(3), msg("Is the mouse turned on?"))]
    MouseOff(),
    #[termination(exit_code(4), msg("No response."))]
    NoResponse(),
    #[termination(exit_code(5), msg("Unknown response: {0:?} with length: {1}"))]
    UnknownResponse([u8; 8], usize),
}

pub struct Device {
    hid_devices: Vec<HidDevice>,
    pub battery_level: u8,
    pub charging: bool,
}
impl Device {
    pub fn new() -> Result<Self, DeviceError> {
        let hid_api = HidApi::new()?;
        let hid_devices: Vec<HidDevice> = hid_api.device_list().filter_map(|info| {
            if PRODUCT_IDS.contains(&info.product_id()) && VENDOR_IDS.contains(&info.vendor_id()) && info.interface_number() == INTERFACE_NUMBER {
                info.open_device(&hid_api).ok()
            } else {
                None
            }
        }).collect();
        if hid_devices.is_empty() {
            return Err(DeviceError::NoDeviceFound());
        }
        Ok(Device {
            hid_devices,
            charging: false,
            battery_level: 0,
        })
    }
    pub fn update_battery_state(&mut self) -> Result<(u8, bool), DeviceError> {
        let mut last_error: Option<DeviceError> = None;
        for hid_device in &self.hid_devices {
            match Self::query_one(hid_device) {
                Ok((level, charging)) => {
                    self.battery_level = level;
                    self.charging = charging;
                    return Ok((level, charging));
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }
        Err(last_error.unwrap_or(DeviceError::NoDeviceFound()))
    }
    fn query_one(hid_device: &HidDevice) -> Result<(u8, bool), DeviceError> {
        hid_device.write(&BATTERY_PACKET)?;
        let mut buf = [0u8; 8];
        let res = hid_device.read_timeout(&mut buf[..], 1000)?;
        if res > RESPONSE_LENGTH && buf.starts_with(&BATTERY_LEVEL_PREAMBLE) {
            Ok(get_battery_state(buf[BATTERY_LEVEL_INDEX]))
        } else if res > RESPONSE_LENGTH && buf.starts_with(&MOUSE_OFF) {
            Err(DeviceError::MouseOff())
        } else {
            Err(DeviceError::UnknownResponse(buf, res))
        }
    }
}
