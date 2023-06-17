use hidapi::{HidApi, HidDevice, HidError};
use thiserror::Error;

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
    (((byte & !charging_flag) - 1) * 5, byte & charging_flag != 0)
}

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("{0}")]
    HidError(#[from] HidError),
    #[error("No device found.")]
    NoDeviceFound(),
    #[error("Is the mouse turned on?")]
    MouseOff(),
    #[error("No response.")]
    NoResponse(),
    #[error("Unknown response: {0:?} with length: {1}")]
    UnknownResponse([u8; 8], usize),
}
   
pub struct Device {
    hid_device: HidDevice,
    pub battery_level: u8,
    pub charging: bool,
}

impl Device {
    pub fn new() -> Result<Self, DeviceError> {
        let hid_api = HidApi::new()?;
        let hid_device = hid_api.device_list().find_map(|info| {
            if PRODUCT_IDS.contains(&info.product_id()) && VENDOR_IDS.contains(&info.vendor_id()) && info.interface_number() == INTERFACE_NUMBER {
                Some(info.open_device(&hid_api))
            } else {
                None
            }
        }).ok_or(DeviceError::NoDeviceFound())??;
        Ok(Device { 
            hid_device,
            charging: false,
            battery_level: 0,
         })
    }

    pub fn update_battery_state(&mut self) -> Result<(u8, bool), DeviceError> {
        self.hid_device.write(&BATTERY_PACKET)?;
        let mut buf = [0u8; 8];
        let res = self.hid_device.read_timeout(&mut buf[..], 1000)?;
        if res > RESPONSE_LENGTH && buf.starts_with(&BATTERY_LEVEL_PREAMBLE) {
            (self.battery_level, self.charging) = get_battery_state(buf[BATTERY_LEVEL_INDEX]);
        } else if res > RESPONSE_LENGTH && buf.starts_with(&MOUSE_OFF) {
            return Err(DeviceError::MouseOff());
        } else {
            return Err(DeviceError::UnknownResponse(buf, res));
        }
        Ok((self.battery_level, self.charging))
    }
}