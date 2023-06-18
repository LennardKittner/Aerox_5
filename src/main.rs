use std::time::Duration;
use std::thread;
use clap::Parser;
use notify_rust::{Notification};
use aerox_5::{Device, DeviceError};
mod battery_tray;
use crate::battery_tray::{TrayHandler, BatteryTray};


fn validate_bounds_0_100(value: &str) -> Result<u8, String> {
    let msg = "The value has to be an integer between 0 and 100.";
    let value = value.parse().map_err(|_| msg)?;
    if value > 100 {
        Err(msg.to_string())
    } else {
        Ok(value)
    }
}

/// A tray application to monitor SteelSeries Aerox 5 Wireless battery level. 
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Enable low-battery desktop notifications.
    #[arg(long, default_value_t = false)]
    enable_notifications: bool,
    /// Set how long the notification will stay on the screen; the notification won't disappear automatically if set to 0.
    #[arg(long, default_value_t = 5)]
    notification_timeout: i32,
    /// Set the battery level below which the notification will be sent.
    #[arg(long, default_value_t = 10, value_parser = validate_bounds_0_100)]
    lower_battery_level: u8,
    /// Set the battery level above which notifications are reenabled.
    #[arg(long, default_value_t = 10, value_parser = validate_bounds_0_100)]
    upper_battery_level: u8,
}

fn pair_device() -> Device {
    loop {
        match Device::new() {
            Ok(device) => break device,
            Err(error) => {
                eprintln!("{error}");
            }
        };
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn handle_error(error: DeviceError, device: &mut Device, tray_handler: &mut TrayHandler) {
    match error {
        DeviceError::HidError(hidapi::HidError::HidApiError { message }) => {
            if message == "No such device" {
                eprintln!("No device found.");
                tray_handler.set_status("No device found.");
                *device = pair_device();
            } else {
                eprintln!("{message}");
            }
        }
        DeviceError::NoDeviceFound() => {
            eprintln!("{}", DeviceError::NoDeviceFound());
            tray_handler.set_status( &DeviceError::NoDeviceFound().to_string());
        }
        DeviceError::MouseOff() => {
            eprintln!("{}", DeviceError::MouseOff());
            tray_handler.set_status(&DeviceError::MouseOff().to_string());
        }
        error => {
            eprintln!("{error}");
        }
    }
}

fn main() {
    let mut tray_handler = TrayHandler::new(BatteryTray::new());
    let mut device = pair_device();
    tray_handler.update(&device);

    // Run loop
    loop {
        match device.update_battery_state() {
            Ok(_) => {
                tray_handler.clear_status();
                tray_handler.update(&device);
            },
            Err(error) => {
                handle_error(error, &mut device, &mut tray_handler);
                thread::sleep(Duration::from_secs(10));
                continue;
            },
        };
        thread::sleep(Duration::from_secs(10));
    }
}
