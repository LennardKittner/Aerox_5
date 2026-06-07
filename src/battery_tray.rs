use std::sync::mpsc::Sender;
use aerox_5::Device;
use ksni::{menu::StandardItem, MenuItem, Tray, ToolTip};
use ksni::blocking::{Handle, TrayMethods};

pub enum TrayMessage {
    ShowSettings,
}

pub struct BatteryTray {
    battery_level: u8,
    charging: bool,
    status_message: Option<String>,
    tx: Sender<TrayMessage>,
}

impl std::fmt::Debug for BatteryTray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BatteryTray")
            .field("battery_level", &self.battery_level)
            .field("charging", &self.charging)
            .field("status_message", &self.status_message)
            .finish()
    }
}

impl BatteryTray {
    pub fn new(tx: Sender<TrayMessage>) -> Self {
        BatteryTray {
            battery_level: 0,
            charging: false,
            status_message: Some("No device found".to_string()),
            tx,
        }
    }

    pub fn set_status(&mut self, message: &str) {
        self.status_message = Some(message.to_string());
    }
}

impl Tray for BatteryTray {
    fn id(&self) -> String {
        "aerox_5".into()
    }

    fn icon_name(&self) -> String {
        "input-mouse".into()
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let tx = self.tx.clone();
        vec![
            StandardItem {
                label: "Settings".into(),
                icon_name: "preferences-system".into(),
                activate: Box::new(move |_| {
                    let _ = tx.send(TrayMessage::ShowSettings);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Exit".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|_| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        ]
    }

    fn tool_tip(&self) -> ToolTip {
        let description = match &self.status_message {
            Some(m) => m.clone(),
            None => {
                let mut s = format!("Battery level: {}%", self.battery_level);
                if self.charging { s += "\nCharging"; } else { s += "\nNot charging"; }
                s
            }
        };
        ToolTip {
            title: "SteelSeries Aerox 5 Wireless".to_string(),
            description,
            icon_name: "".into(),
            icon_pixmap: Vec::new(),
        }
    }
}

pub struct TrayHandler {
    handle: Handle<BatteryTray>,
}

impl Clone for TrayHandler {
    fn clone(&self) -> Self {
        TrayHandler { handle: self.handle.clone() }
    }
}

impl TrayHandler {
    pub fn new(tray: BatteryTray) -> Self {
        let handle = tray.spawn().expect("failed to start tray service");
        TrayHandler { handle }
    }

    pub fn update(&self, device: &Device) {
        let level = device.battery_level;
        let charging = device.charging;
        self.handle.update(move |tray: &mut BatteryTray| {
            tray.battery_level = level;
            tray.charging = charging;
        });
    }

    pub fn clear_status_and_update(&self, device: &Device) {
        let level = device.battery_level;
        let charging = device.charging;
        self.handle.update(move |tray: &mut BatteryTray| {
            tray.status_message = None;
            tray.battery_level = level;
            tray.charging = charging;
        });
    }

    pub fn set_status(&self, message: &str) {
        let msg = message.to_string();
        self.handle.update(move |tray: &mut BatteryTray| tray.set_status(&msg));
    }
}
