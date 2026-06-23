# Aerox_5
A CLI and tray application to monitor SteelSeries Aerox 5 Wireless battery level. 

<img src=./screenshots/tray_app.png alt="tray_app">
<img src=./screenshots/notification.png alt="notification">
<img width="702" height="430" alt="Schermata_20260611_173233" src="https://github.com/user-attachments/assets/546cecaa-011c-4d19-8a8e-a5cb7f41f438" />


## Compatibility
The CLI application is compatible with both Linux and MacOS operating systems. However, the tray application is only functional on Linux. Although it was only tested on Manjaro/KDE, it should also work on other distribution and desktop environments.

Currently, only the SteelSeries Aerox 5 Wireless is supported.

## Prerequisites

### Hidraw

Make sure you have hidraw installed on your system.

Debian/Ubuntu:

`sudo apt install libhidapi-hidraw0`

Arch:

`sudo pacman -S hidapi`

MacOS:

`brew install hidapi`

### Other Dependencies

These dependencies are probably already installed.

Debian/Ubuntu:

`sudo apt install libdbus-1-dev libusb-1.0-0-dev libudev-dev`

Arch:

`sudo pacman -S dbus libusb`

MacOS:

`brew install libusb`

### Udev (Linux only)

Create a new file in /etc/udev/rules.d/99-aerox-5.rules with the following content inside:

```
SUBSYSTEMS=="usb", ATTRS{idProduct}=="185E", ATTRS{idVendor}=="1038", MODE="0666"
SUBSYSTEMS=="usb", ATTRS{idProduct}=="1862", ATTRS{idVendor}=="1038", MODE="0666"
SUBSYSTEMS=="usb", ATTRS{idProduct}=="1852", ATTRS{idVendor}=="1038", MODE="0666"
SUBSYSTEMS=="usb", ATTRS{idProduct}=="185C", ATTRS{idVendor}=="1038", MODE="0666"
SUBSYSTEMS=="usb", ATTRS{idProduct}=="1860", ATTRS{idVendor}=="1038", MODE="0666"
SUBSYSTEMS=="usb", ATTRS{idProduct}=="1854", ATTRS{idVendor}=="1038", MODE="0666"

KERNEL=="hidraw*", ATTRS{idProduct}=="185E", ATTRS{idVendor}=="1038", MODE="0666"
KERNEL=="hidraw*", ATTRS{idProduct}=="1862", ATTRS{idVendor}=="1038", MODE="0666"
KERNEL=="hidraw*", ATTRS{idProduct}=="1852", ATTRS{idVendor}=="1038", MODE="0666"
KERNEL=="hidraw*", ATTRS{idProduct}=="185C", ATTRS{idVendor}=="1038", MODE="0666"
KERNEL=="hidraw*", ATTRS{idProduct}=="1860", ATTRS{idVendor}=="1038", MODE="0666"
KERNEL=="hidraw*", ATTRS{idProduct}=="1854", ATTRS{idVendor}=="1038", MODE="0666"
```

Once created, replug the wireless dongle.

## Building

To only build the cli_app on MacOS, use:
`cargo build --release --bin cli_app`

To build both applications on Linux, use:
`cargo build --release`

You can also download a compiled version from [releases](https://github.com/LennardKittner/Aerox_5/releases).

`cargo build --release` **will fail on MacOS** because cargo will try to build the tray application, but some dependencies are exclusive to Linux.

## Usage

`cli_app` without any arguments will print the current battery level and whether the device is charging.

`aerox_5` without any arguments will start the tray application. Hover over the icon in the system tray to see the battery level. Right-click to exit.

### Notifications

The tray application supports two independent notification types:

**Low-battery notifications** — enabled with `--enable-notifications`, fires when the battery drops below a threshold:

```
--enable-notifications                     Enable low-battery desktop notifications
--lower-battery-level <LEVEL>              Battery % below which the notification is sent [default: 10]
--upper-battery-level <LEVEL>              Battery % above which the notification is re-enabled [default: 10]
--notification-timeout-in-seconds <SECS>   How long the notification stays on screen; 0 = no auto-dismiss [default: 5]
```

**Full-charge notifications** — enabled with `--full-charge-level`, fires when the battery reaches the specified level while charging. Works independently of `--enable-notifications`:

```
--full-charge-level <LEVEL>   Battery % at which a "charged" notification is sent while charging
```

### Examples

```bash
# Low-battery notification when below 15%, re-enable above 20%
aerox_5 --enable-notifications --lower-battery-level 15 --upper-battery-level 20

# Notify when fully charged (100%)
aerox_5 --full-charge-level 100

# Notify when charging reaches 80% (useful for battery health)
aerox_5 --full-charge-level 80

# Both low-battery and full-charge notifications
aerox_5 --enable-notifications --full-charge-level 100
```

## TODOs
- [ ] Menu bar app for MacOS.

## Other Projects

The device packets were taken from [rivalcfg](https://github.com/flozz/rivalcfg).
