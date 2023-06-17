use std::ffi::{CString, CStr};

use aerox_5::Device;
use hidapi::HidApi;

fn main() {
    // let hid_api = HidApi::new().unwrap();
    // // let device = hid_api.open(0x1038, 0x1852).unwrap();
    // let mut cs: &CStr = &CString::new("").unwrap();
    // for d in hid_api.device_list().enumerate() {
    //     if d.1.vendor_id() == 0x1038 && d.1.interface_number() == 3 {
    //         cs = d.1.path();
    //     }
    // }
    // let device = hid_api.open_path(&cs).unwrap();
    // println!("{}",device.get_device_info().unwrap().interface_number());
    // println!("{}", device.get_manufacturer_string().unwrap().unwrap());
    // let mut send = [0u8, 0u8];
    // send[1] = 0xd2;
    // loop {
    //     std::thread::sleep(std::time::Duration::from_secs(1));
    //     if let Ok(size) = device.write(&mut send) {
    //         println!("{size}");
    //         let mut buf = [0u8; 2];
    //         let size = device.read_timeout(&mut buf, 10000).unwrap();
    //         println!("{:?}", &buf[..size]);
    //     } else {
    //         println!("fail")
    //     }
    // }
    let mut device = match Device::new() {
        Ok(device) => device,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    let battery_level = match device.update_battery_state() {
        Ok(t) => t,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    println!("Battery level: {}% {}", battery_level.0, battery_level.1);
}

#[test]
fn test_basic_device_access() {
    let _ = match Device::new() {
        Ok(device) => device,
        Err(_) => return
    };
}