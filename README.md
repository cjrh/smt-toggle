# smt-toggle

A GUI application with system tray integration for toggling CPU SMT (Simultaneous Multi-Threading / Hyperthreading) on Linux.

## Features

- Toggle SMT on/off via a simple GUI toggle switch
- System tray icon - closing the window minimizes to tray
- Automatic privilege escalation via pkexec when needed
- Displays current SMT status (on, off, forceoff, notsupported)

## How it Works

The application reads and writes to `/sys/devices/system/cpu/smt/control` to control SMT. If direct write fails (no root permissions), it uses `pkexec tee` to prompt for authentication.

## System Requirements

- **Linux** (uses Linux sysfs interface)
- **GTK3** development libraries
- **libappindicator3** or **libayatana-appindicator3** (for system tray)
- **libxdo** (required by tray-icon crate)
- **pkexec** (polkit) for privilege escalation
- **Rust** toolchain (edition 2024)

### Fedora/RHEL

```bash
sudo dnf install gtk3-devel libappindicator-gtk3-devel libxdo-devel
```

### Ubuntu/Debian

```bash
sudo apt install libgtk-3-dev libayatana-appindicator3-dev libxdo-dev
```

### Arch

```bash
sudo pacman -S gtk3 libappindicator-gtk3 xdotool
```

## Building

```bash
cargo build --release
```

The binary will be at `target/release/smt-toggle`.

## Usage

```bash
./target/release/smt-toggle
```

The application starts with a window showing the current SMT status. Use the toggle to enable/disable SMT. Closing the window hides it to the system tray - use the tray icon menu to show the window again or quit the application.

## Dependencies

- [iced](https://github.com/iced-rs/iced) 0.14 - GUI framework
- [tray-icon](https://github.com/tauri-apps/tray-icon) 0.21.3 - System tray
- [gtk](https://gtk-rs.org/) 0.18 - GTK bindings (for tray support)
- [tokio](https://tokio.rs/) 1 - Async runtime

## License

MIT
