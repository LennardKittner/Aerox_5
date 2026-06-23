use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button,
    Grid, Label, Orientation, Separator, SpinButton, Switch,
};
use std::sync::{Arc, Mutex};
use crate::config::{Config, is_autostart_enabled, set_autostart};

pub fn show_settings_window(app: &Application, config: Arc<Mutex<Config>>, is_first_run: bool) {
    for window in app.windows() {
        if window.title().as_deref() == Some("Aerox 5 \u{2013} Settings") {
            window.present();
            return;
        }
    }

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Aerox 5 \u{2013} Settings")
        .default_width(460)
        .resizable(false)
        .build();

    let vbox = GtkBox::new(Orientation::Vertical, 16);
    vbox.set_margin_top(20);
    vbox.set_margin_bottom(20);
    vbox.set_margin_start(20);
    vbox.set_margin_end(20);

    if is_first_run {
        let welcome = Label::new(Some("Welcome! Configure your Aerox 5 preferences below."));
        welcome.set_halign(Align::Start);
        vbox.append(&welcome);
        vbox.append(&Separator::new(Orientation::Horizontal));
    }

    let grid = Grid::new();
    grid.set_column_spacing(16);
    grid.set_row_spacing(10);

    let notif_label = Label::new(Some("Enable low battery notifications"));
    notif_label.set_halign(Align::Start);
    notif_label.set_hexpand(true);
    let notif_switch = Switch::new();
    notif_switch.set_valign(Align::Center);
    notif_switch.set_halign(Align::End);
    grid.attach(&notif_label, 0, 0, 1, 1);
    grid.attach(&notif_switch, 1, 0, 1, 1);

    let timeout_label = Label::new(Some("Notification timeout (seconds, 0 = persistent)"));
    timeout_label.set_halign(Align::Start);
    let timeout_spin = SpinButton::with_range(0.0, 60.0, 1.0);
    grid.attach(&timeout_label, 0, 1, 1, 1);
    grid.attach(&timeout_spin, 1, 1, 1, 1);

    let lower_label = Label::new(Some("Notify below (%)"));
    lower_label.set_halign(Align::Start);
    let lower_spin = SpinButton::with_range(1.0, 99.0, 1.0);
    grid.attach(&lower_label, 0, 2, 1, 1);
    grid.attach(&lower_spin, 1, 2, 1, 1);

    let upper_label = Label::new(Some("Re-enable notifications above (%)"));
    upper_label.set_halign(Align::Start);
    let upper_spin = SpinButton::with_range(1.0, 100.0, 1.0);
    grid.attach(&upper_label, 0, 3, 1, 1);
    grid.attach(&upper_spin, 1, 3, 1, 1);

    let full_label = Label::new(Some("Notify when charging reaches (%)"));
    full_label.set_halign(Align::Start);
    let full_switch = Switch::new();
    full_switch.set_valign(Align::Center);
    full_switch.set_halign(Align::End);
    grid.attach(&full_label, 0, 4, 1, 1);
    grid.attach(&full_switch, 1, 4, 1, 1);

    let full_target_label = Label::new(Some("Target level (%)"));
    full_target_label.set_halign(Align::Start);
    let full_spin = SpinButton::with_range(1.0, 100.0, 1.0);
    grid.attach(&full_target_label, 0, 5, 1, 1);
    grid.attach(&full_spin, 1, 5, 1, 1);

    {
        let cfg = config.lock().unwrap();
        notif_switch.set_active(cfg.enable_notifications);
        timeout_spin.set_value(cfg.notification_timeout_in_seconds as f64);
        lower_spin.set_value(cfg.lower_battery_level as f64);
        upper_spin.set_value(cfg.upper_battery_level as f64);
        if let Some(level) = cfg.full_charge_level {
            full_switch.set_active(true);
            full_spin.set_value(level as f64);
        } else {
            full_switch.set_active(false);
            full_spin.set_value(100.0);
            full_spin.set_sensitive(false);
        }
    }

    full_switch.connect_active_notify(glib::clone!(
        #[weak] full_spin,
        move |sw| { full_spin.set_sensitive(sw.is_active()); }
    ));

    vbox.append(&grid);
    vbox.append(&Separator::new(Orientation::Horizontal));

    let autostart_hbox = GtkBox::new(Orientation::Horizontal, 12);
    let autostart_vbox = GtkBox::new(Orientation::Vertical, 2);
    let autostart_label = Label::new(Some("Enable XDG autostart"));
    autostart_label.set_halign(Align::Start);
    let autostart_sublabel = Label::new(Some("Start automatically on login (requires XDG autostart support, e.g. GNOME, KDE)"));
    autostart_sublabel.set_halign(Align::Start);
    autostart_sublabel.add_css_class("dim-label");
    autostart_vbox.append(&autostart_label);
    autostart_vbox.append(&autostart_sublabel);
    autostart_vbox.set_hexpand(true);
    let autostart_switch = Switch::new();
    autostart_switch.set_valign(Align::Center);
    autostart_switch.set_active(is_autostart_enabled());
    autostart_hbox.append(&autostart_vbox);
    autostart_hbox.append(&autostart_switch);
    vbox.append(&autostart_hbox);

    vbox.append(&Separator::new(Orientation::Horizontal));

    let btn_box = GtkBox::new(Orientation::Horizontal, 8);
    btn_box.set_halign(Align::End);
    let cancel_btn = Button::with_label("Cancel");
    let save_btn = Button::with_label("Save");
    save_btn.add_css_class("suggested-action");
    btn_box.append(&cancel_btn);
    btn_box.append(&save_btn);
    vbox.append(&btn_box);

    window.set_child(Some(&vbox));

    cancel_btn.connect_clicked(glib::clone!(
        #[weak] window,
        move |_| { window.close(); }
    ));

    let config_for_save = config.clone();
    save_btn.connect_clicked(glib::clone!(
        #[weak] window,
        #[weak] notif_switch,
        #[weak] timeout_spin,
        #[weak] lower_spin,
        #[weak] upper_spin,
        #[weak] full_switch,
        #[weak] full_spin,
        #[weak] autostart_switch,
        move |_| {
            let new_config = Config {
                enable_notifications: notif_switch.is_active(),
                notification_timeout_in_seconds: timeout_spin.value() as i32,
                lower_battery_level: lower_spin.value() as u8,
                upper_battery_level: upper_spin.value() as u8,
                full_charge_level: if full_switch.is_active() {
                    Some(full_spin.value() as u8)
                } else {
                    None
                },
            };
            *config_for_save.lock().unwrap() = new_config.clone();
            if let Err(e) = new_config.save() {
                eprintln!("Failed to save config: {e}");
            }
            if let Err(e) = set_autostart(autostart_switch.is_active()) {
                eprintln!("Failed to set autostart: {e}");
            }
            window.close();
        }
    ));

    window.present();
}
