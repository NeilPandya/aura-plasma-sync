# openrgb-accent-sync

Synchronize your XDG-compliant desktop environment's accent color with any **OpenRGB-compatible lighting hardware** via the **OpenRGB SDK Server**.

---

## Table of Contents
- [Features](#features)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
  - [Systemd Service (Recommended)](#systemd-service-recommended)
  - [Manual Build & Install](#manual-build--install)
- [Usage](#usage)
- [Troubleshooting](#troubleshooting)
- [Development](#development)
  - [Running Checks](#running-checks)
  - [Building](#building)
  - [Testing](#testing)
- [License](#license)
- [Contributing](#contributing)

---

## Features
- **Pure XDG Compliance**: Uses only the `org.freedesktop.portal.Settings` interface for accent color detection
- **Real-time Updates**: Listens for `SettingChanged` signals for instant color synchronization
- **Cross-Desktop Compatibility**: Works with any XDG-compatible desktop environment including:
  - GNOME 47+
  - KDE Plasma 5.24+
  - Cosmic (System76)
  - Pantheon (elementary OS)
  - And any other DE that implements the XDG Settings Portal
- **Systemd Integration**: Includes an installer for persistent background service
- **Tray Integration**: Provides a system tray icon with toggle functionality

---

## Prerequisites

- **Rust**: Installed via [rustup](https://rust-lang.github.io/rustup/)
- **OpenRGB**: Version 0.9 or later with **SDK Server enabled**. 
  - Download: [openrgb.io](https://openrgb.io)
  - Verify SDK server is listening: `nc -zv 127.0.0.1 6742` (default port)
  - Check connected devices: Run OpenRGB and view the device list in the GUI
- **XDG Settings Portal**: Available in modern desktop environments
- **systemd** (user-level): For running the service (recommended)

Required Rust crates:
- `zbus` – DBus communication with XDG Portal
- `anyhow` – Error handling
- `clap` – Command-line parsing
- `tray-icon` – Cross-platform system tray
- `which` – Dependency validation
- `log` / `env_logger` – Logging

---

## Installation

### Systemd Service (Recommended)
The project ships an installer that creates a **user‑level systemd service** pointing to the binary you just built.

```bash
git clone https://codeberg.org/neilpandya/openrgb-accent-sync
openrgb-accent-sync install
cargo install --path .
aura-plasma-sync install
```

The installer:
1. Detects the current executable path.
2. Writes a unit file into `~/.config/systemd/user/aura-plasma-sync.service`.
3. Reloads the systemd daemon.
4. Enables and starts the service immediately.

To verify:
```bash
systemctl --user status openrgb-accent-sync.service
```

### Manual Build & Install
```bash
git clone https://codeberg.org/neilpandya/openrgb-accent-sync
cd openrgb-accent-sync
cargo build --release
sudo cp target/release/openrgb-accent-sync /usr/local/bin/
```

#### Custom Host/Port Configuration

By default, the utility connects to the OpenRGB SDK server at `127.0.0.1:6742`. To use a different host or port (e.g., for remote OpenRGB instances):

```bash
# During installation
openrgb-accent-sync install --host <your-host-ip-here> --port <your-port-here>

# Or manually edit the systemd unit
systemctl --user edit openrgb-accent-sync.service
# Modify the ExecStart= line with your desired --host and --port flags
systemctl --user daemon-reload
systemctl --user restart openrgb-accent-sync
```

---

## Usage

The utility works in the background; you do not need to interact with it directly.

When the service is running, a tray icon appears in your desktop panel.

- **Colored icon** – Displays the current accent color.
- **Tray menu** – Shows the current HEX and RGB values of the synced color.

To stop or disable later:
```bash
openrgb-accent-sync uninstall
```

---

## Troubleshooting

### Known Issues

#### Ayatana AppIndicator Deprecation Warning

When running:
```bash
journalctl --user -u aura-accent-sync -f
```
You may see a warning like:
```log
libayatana-appindicator is deprecated. Please use libayatana-appindicator-glib in newly written code.
```

This warning originates from underlying system libraries used by the tray icon implementation. It's purely cosmetic and doesn't affect functionality. The `tray-icon` crate maintainers are aware of this and will address it in future updates.

#### Missing `libxdo.so.3` Error

If you encounter an error like:
```bash
error while loading shared libraries: libxdo.so.3: cannot open shared object file: No such file or directory
```

This typically happens when the system library `libxdo` has been updated. To fix this:

1. **Rebuild the application** (recommended):
   ```bash
   cargo clean
   cargo install --path .
   ```
   or
   ```bash
   cargo clean
   cargo build --release
   ./target/release/aura-accent-sync install
   ```

2. **Install the missing library**:
  e.g.,
   ```bash
   sudo apt-get install libxdo-dev
   ```
   or
   ```
   ```fish
   paru -S xdotool  # On Arch/CachyOS
   ```
   or
   ```sh
   sudo pacman -S xdotool  # On Arch/CachyOS
   ```

This issue occurs because the binary was compiled against a specific version of system libraries that may have been updated by your package manager.

#### `Failed to connect to OpenRGB SDK server`:

1. **Verify OpenRGB is running:**
   ```bash
   pgrep -a openrgb  # Should list the OpenRGB process
   ```

2. **Check SDK Server is enabled:**
   - Open OpenRGB GUI → Settings → General
   - Ensure "Start Server" is checked
   - Default port should be `6742`

3. **Test connectivity:**
   ```bash
   nc -zv 127.0.0.1 6742
   ```
   If connection fails, the SDK server is not listening.

4. **Verify protocol version:**
   - openrgb-accent-sync requires **Protocol v4** (OpenRGB 0.9+)
   - Check your OpenRGB version: `openrgb --version` or GUI → Help → About

#### No Devices Updating

If the color syncs but no LEDs change:

1. **Check OpenRGB can see devices:**
   - Open OpenRGB GUI → Device List
   - At least one device should be listed

2. **Verify device mode:**
   - In OpenRGB GUI, click each device
   - Set mode to **"Direct"** or **"Static"** (not "Disabled")
   - `openrgb-accent-sync` sends direct color commands

3. **Check logs for errors:**
   ```bash
   journalctl --user -u openrgb-accent-sync -f
   ```
   Look for lines like `Failed to queue LED update` or `No controllers had LEDs to update`

#### Color Not Syncing Immediately

If the accent color changes in your DE but the LEDs don't update:

1. **Verify XDG Portal is working:**
   ```bash
   gdbus call --session --dest org.freedesktop.portal.Desktop \
   --object-path /org/freedesktop/portal/desktop \
   --method org.freedesktop.portal.Settings.Read \
   org.freedesktop.appearance accent-color
   ```

2. **Check service is running:**
   ```bash
   systemctl --user status openrgb-accent-sync.service
   ```

3. **Enable debug logging:**
   ```bash
   RUST_LOG=debug journalctl --user -u openrgb-accent-sync -f
   ```
   Look for `Parsed color:` and `Hardware Updated:` lines to confirm the sync cycle completed.

---

## Development

### Running Checks
```bash
cargo fmt -- --check
cargo clippy -- -D warnings
```

### Building
```bash
cargo build --release
```

### Testing
The project contains no unit tests yet, but you can add them under `src/` as `tests/` or `#[cfg(test)]` modules later. CI pipelines (e.g., GitHub Actions) can be set up to run `cargo fmt`, `cargo clippy`, and `cargo test` on every push.

---

## License
Distributed under the **GNU General Public License v3.0 or later**.  
See the `LICENSE` file for more information.

---

## Contributing

This project recently transitioned from Asus-specific (AniMatrix) to universal OpenRGB support. 

We welcome contributions such as:
- Bug reports and fixes
- Documentation improvements
- Support for additional OpenRGB features (e.g., device-specific profiles)
- Testing and debugging
- CI/CD pipeline setup (GitHub Actions)


### Contributing Guidelines

Contributions are welcome! Please:

1. Fork the repository.
2. Create a feature branch (`git checkout -b my-feature`).
3. Submit a pull request with a concise description of the change.
4. Ensure code follows Rust idioms and passes `cargo fmt` & `cargo clippy`.

**Code of Conduct** – By participating you agree to follow the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/version/3/0/code_of_conduct/).

---

*Enjoy perfectly synced lighting across all your OpenRGB-compatible hardware and desktop environment!*
