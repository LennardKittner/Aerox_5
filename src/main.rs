use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

use clap::Parser;
use gtk4::prelude::*;
use gtk4::Application;
use notify_rust::Notification;
use aerox_5::{Device, DeviceError};

mod battery_tray;
mod config;
mod gui;

use battery_tray::{BatteryTray, TrayHandler, TrayMessage};
use config::Config;
use gui::show_settings_window;

fn validate_bounds_0_100(value: &str) -> Result<u8, String> {
    let msg = "The value has to be an integer between 0 and 100.";
    let value: u8 = value.parse().map_err(|_| String::from(msg))?;
    if value > 100 { Err(msg.to_string()) } else { Ok(value) }
}

/// Tray application to monitor SteelSeries Aerox 5 Wireless battery level.
/// CLI flags override the saved config for this session only.
#[derive(Parser, Debug)]
#[command(version, after_help = "\
Examples:
  # Low-battery notification when below 15%, re-enable above 20%
  aerox_5 --enable-notifications --lower-battery-level 15 --upper-battery-level 20

  # Notify when fully charged (100%)
  aerox_5 --full-charge-level 100

  # Both low-battery and full-charge notifications
  aerox_5 --enable-notifications --full-charge-level 100")]
struct Args {
    /// Enable low-battery desktop notifications.
    #[arg(long)]
    enable_notifications: bool,
    /// How long the notification stays on screen (0 = persistent).
    #[arg(long, value_parser = |s: &str| s.parse::<i32>().map_err(|e| e.to_string()))]
    notification_timeout_in_seconds: Option<i32>,
    /// Battery level below which the low-battery notification is sent. Requires --enable-notifications.
    #[arg(long, value_parser = validate_bounds_0_100)]
    lower_battery_level: Option<u8>,
    /// Battery level above which the low-battery notification is re-enabled. Requires --enable-notifications.
    #[arg(long, value_parser = validate_bounds_0_100)]
    upper_battery_level: Option<u8>,
    /// Notify when battery reaches this level while charging. If not set, no full-charge notification is sent.
    #[arg(long, value_parser = validate_bounds_0_100)]
    full_charge_level: Option<u8>,
}

impl Args {
    fn apply_to(&self, config: &mut Config) {
        if self.enable_notifications {
            config.enable_notifications = true;
        }
        if let Some(v) = self.notification_timeout_in_seconds {
            config.notification_timeout_in_seconds = v;
        }
        if let Some(v) = self.lower_battery_level {
            config.lower_battery_level = v;
        }
        if let Some(v) = self.upper_battery_level {
            config.upper_battery_level = v;
        }
        if self.full_charge_level.is_some() {
            config.full_charge_level = self.full_charge_level;
        }
    }
}

fn pair_device() -> Device {
    loop {
        match Device::new() {
            Ok(device) => break device,
            Err(error) => eprintln!("{error}"),
        }
        thread::sleep(Duration::from_secs(1));
    }
}

fn handle_error(error: DeviceError, device: &mut Device, tray: &TrayHandler) {
    match error {
        DeviceError::HidError(hidapi::HidError::HidApiError { message }) => {
            if message == "No such device" {
                eprintln!("No device found.");
                tray.set_status("No device found.");
                *device = pair_device();
                if let Ok(_) = device.update_battery_state() {
                    tray.clear_status_and_update(device);
                }
            } else {
                eprintln!("{message}");
            }
        }
        DeviceError::NoDeviceFound() => {
            eprintln!("{}", DeviceError::NoDeviceFound());
            tray.set_status(&DeviceError::NoDeviceFound().to_string());
        }
        DeviceError::MouseOff() => {
            eprintln!("{}", DeviceError::MouseOff());
            tray.set_status(&DeviceError::MouseOff().to_string());
        }
        error => eprintln!("{error}"),
    }
}

fn battery_loop(config: Arc<Mutex<Config>>, tray: TrayHandler) {
    let mut device = pair_device();
    tray.update(&device);
    let mut notification_blocked = false;
    let mut fully_charged_notified = false;

    loop {
        let (battery_level, charging) = match device.update_battery_state() {
            Ok(t) => {
                tray.clear_status_and_update(&device);
                t
            }
            Err(error) => {
                handle_error(error, &mut device, &tray);
                thread::sleep(Duration::from_secs(5));
                continue;
            }
        };

        let cfg = config.lock().unwrap().clone();

        if cfg.enable_notifications && !notification_blocked && battery_level <= cfg.lower_battery_level {
            if let Err(e) = Notification::new()
                .summary("SteelSeries Aerox 5 Wireless")
                .body(&format!("Battery level low!\n{}% remaining", battery_level))
                .icon("input-mouse")
                .appname("Aerox 5")
                .timeout(Duration::from_secs(cfg.notification_timeout_in_seconds as u64))
                .show()
            {
                eprintln!("{e}");
            } else {
                notification_blocked = true;
            }
        } else if cfg.enable_notifications && battery_level > cfg.upper_battery_level {
            notification_blocked = false;
        }


        if let Some(full_charge_level) = cfg.full_charge_level {
            if charging && battery_level >= full_charge_level && !fully_charged_notified {
                if let Err(e) = Notification::new()
                    .summary("SteelSeries Aerox 5 Wireless")
                    .body(&format!("Battery reached {}% while charging!", battery_level))
                    .icon("input-mouse")
                    .appname("Aerox 5")
                    .timeout(Duration::from_secs(cfg.notification_timeout_in_seconds as u64))
                    .show()
                {
                    eprintln!("{e}");
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

fn main() {
    let args = Args::parse();
    let is_first_run = !Config::exists();
    let mut base_config = Config::load_or_default();
    args.apply_to(&mut base_config);
    let config = Arc::new(Mutex::new(base_config));

    let (tx, rx) = mpsc::channel::<TrayMessage>();

    let app = Application::builder()
        .application_id("io.github.LennardKittner.Aerox_5")
        .build();

    let config_activate = config.clone();
    let rx_cell = Rc::new(RefCell::new(Some(rx)));
    let tray_started = Rc::new(RefCell::new(false));

    app.connect_activate(move |app| {
        std::mem::forget(app.hold());

        if let Some(rx) = rx_cell.borrow_mut().take() {
            let app_timer = app.clone();
            let config_timer = config_activate.clone();
            glib::timeout_add_local(Duration::from_millis(100), move || {
                while let Ok(msg) = rx.try_recv() {
                    match msg {
                        TrayMessage::ShowSettings => {
                            show_settings_window(&app_timer, config_timer.clone(), false);
                        }
                    }
                }
                glib::ControlFlow::Continue
            });
        }

        if !*tray_started.borrow() {
            *tray_started.borrow_mut() = true;

            let tray = BatteryTray::new(tx.clone());
            let handler = TrayHandler::new(tray);
            let handler_bt = handler.clone();
            let config_bt = config_activate.clone();
            thread::spawn(move || battery_loop(config_bt, handler_bt));

            if is_first_run {
                show_settings_window(app, config_activate.clone(), true);
            }
        }
    });

    app.run();
}
