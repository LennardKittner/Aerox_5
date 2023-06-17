use aerox_5::Device;

fn main() {
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
    println!("Battery level: {}% {}", battery_level.0, if battery_level.1 {"Charging"} else {"Discharging"});
}

#[test]
fn test_basic_device_access() {
    let _ = match Device::new() {
        Ok(device) => device,
        Err(_) => return
    };
}