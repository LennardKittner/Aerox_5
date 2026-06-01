# Aerox_5
A CLI and tray application to monitor SteelSeries Aerox 5 Wireless battery level.

<img src=./screenshots/tray_app.png alt="tray_app">
<img src=./screenshots/notification.png alt="notification">

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
`cli_app` without any arguments will print the current battery level and if the device is charging.

```
Usage: aerox_5 [OPTIONS]

Options:
      --enable-notifications
          Enable low-battery desktop notifications.
      --notification-timeout-in-seconds <NOTIFICATION_TIMEOUT_IN_SECONDS>
          Set how long the notification will stay on the screen; the notification won't disappear automatically if set to 0. [default: 5]
      --lower-battery-level <LOWER_BATTERY_LEVEL>
          Set the battery level below which the low-battery notification will be sent. Requires --enable-notifications. [default: 10]
      --upper-battery-level <UPPER_BATTERY_LEVEL>
          Set the battery level above which the low-battery notification is re-enabled. Requires --enable-notifications. [default: 10]
      --full-charge-level <FULL_CHARGE_LEVEL>
          Send a notification when battery reaches this level while charging. Independent of --enable-notifications. If not set, no full-charge notification is sent.
  -h, --help
          Print help
  -V, --version
          Print version
```
`aerox_5` without any arguments will start the tray application. Once it's open, hover over the headset icon in the system tray to view details like the battery level. To exit, right-click on the icon.
The `--enable-notifications` flag will enable notifications when the mouse battery level drops under a threshold value.
The `--full-charge-level` flag enables a separate notification sent when the battery reaches the specified level while charging — useful to know when to unplug. It works independently of `--enable-notifications`.
The other arguments can be used to customize the notification behavior.

Examples:
```bash
# Low-battery notification when below 15%, re-enable above 20%
aerox_5 --enable-notifications --lower-battery-level 15 --upper-battery-level 20

# Notify when fully charged (100%)
aerox_5 --full-charge-level 100

# Notify when charging reaches 80%
aerox_5 --full-charge-level 80

# Both low-battery and full-charge notifications
aerox_5 --enable-notifications --full-charge-level 100
```

## TODOs
- [ ] Menu bar app for MacOS.

## Other Projects

The device packets were taken from [rivalcfg](https://github.com/flozz/rivalcfg).