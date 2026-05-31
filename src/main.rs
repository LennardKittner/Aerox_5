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
#[command(version, after_help = "\
Examples:
  # Low-battery notification when below 15%, re-enable above 20%
  aerox_5 --enable-notifications --lower-battery-level 15 --upper-battery-level 20

  # Notify when fully charged (100%)
  aerox_5 --full-charge-level 100

  # Notify when charging reaches 80%
  aerox_5 --full-charge-level 80

  # Both low-battery and full-charge notifications
  aerox_5 --enable-notifications --full-charge-level 100")]
struct Args {
    /// Enable low-battery desktop notifications.
    #[arg(long, default_value_t = false)]
    enable_notifications: bool,
    /// Set how long the notification will stay on the screen; the notification won't disappear automatically if set to 0.
    #[arg(long, default_value_t = 5)]
    notification_timeout: i32,
    /// Set the battery level below which the low-battery notification will be sent. Requires --enable-notifications.
    #[arg(long, default_value_t = 10, value_parser = validate_bounds_0_100)]
    lower_battery_level: u8,
    /// Set the battery level above which the low-battery notification is re-enabled. Requires --enable-notifications.
    #[arg(long, default_value_t = 10, value_parser = validate_bounds_0_100)]
    upper_battery_level: u8,
    /// Send a notification when battery reaches this level while charging. Independent of --enable-notifications. If not set, no full-charge notification is sent.
    #[arg(long, value_parser = validate_bounds_0_100)]
    full_charge_level: Option<u8>,
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
    let args = Args::parse();

    let mut tray_handler = TrayHandler::new(BatteryTray::new());
    let mut device = pair_device();
    tray_handler.update(&device);

    let mut notification_blocked = false;
    let mut fully_charged_notified = false;
    // Run loop
    loop {
        let (battery_level, charging) = match device.update_battery_state() {
            Ok(t) => {
                tray_handler.clear_status();
                tray_handler.update(&device);
                t
            },
            Err(error) => {
                handle_error(error, &mut device, &mut tray_handler);
                thread::sleep(Duration::from_secs(5));
                continue;
            },
        };
        if args.enable_notifications && !notification_blocked && battery_level <= args.lower_battery_level {
            if let Err(error) = Notification::new()
                .summary("SteelSeries Aerox 5 Wireless")
                .body(&format!("Battery level low!\n{}% remaining", battery_level))
                .icon("input-mouse")
                .appname("Aerox 5")
                .timeout(args.notification_timeout)
                .show() {
                    eprintln!("{error}");
            } else {
                notification_blocked = true;
            }
        } else if args.enable_notifications && battery_level > args.upper_battery_level {
            notification_blocked = false;
        }
        if let Some(full_charge_level) = args.full_charge_level {
            if charging && battery_level >= full_charge_level && !fully_charged_notified {
                if let Err(error) = Notification::new()
                    .summary("SteelSeries Aerox 5 Wireless")
                    .body(&format!("Battery reached {}% while charging!", battery_level))
                    .icon("input-mouse")
                    .appname("Aerox 5")
                    .timeout(args.notification_timeout)
                    .show() {
                        eprintln!("{error}");
                } else {
                    fully_charged_notified = true;
                }
            } else if !charging || battery_level < full_charge_level {
                fully_charged_notified = false;
            }
        }
        thread::sleep(Duration::from_secs(5));
    }
}
