# aura-accent-sync

A lightweight utility that synchronizes **Asus Aura** lighting with your desktop environment's accent color via **`asusctl`** and the **XDG Settings Portal** standard.

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

- **Rust**: Installed via [rustup](https://rust-lang.github.io/rustup/) (required for building from source)
- **Asus Hardware**: Device with Aura RGB lighting supported by `asusctl`
- [**`asusctl`**](https://gitlab.com/asus-linux/asusctl): The utility that controls Asus Aura devices (tested with v6.3.2+)
```bash
asusctl aura effect static -c <hex-color>
```

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
git clone https://github.com/neilpandya/aura-accent-sync
cd aura-accent-sync
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
systemctl --user status aura-plasma-sync.service
```

### Manual Build & Install
```bash
git clone https://github.com/neilpandya/aura-accent-sync
cd aura-accent-sync
cargo build --release
sudo cp target/release/aura-accent-sync /usr/local/bin/
```

To stop or disable later:
```bash
aura-accent-sync uninstall
```

---

Usage
The utility works in the background; you do not need to interact with it directly unless you wish to change the toggle.

When the service is running, a tray icon appears in your desktop panel.

- **Active** (colored icon) – the sync thread is alive and listening for changes.
- **Inactive** (gray icon) – syncing is paused; you can toggle it via the tray menu entry **"Toggle Sync"**.

---

## Troubleshooting

### Known Issues

**Ayatana AppIndicator Deprecation Warning**

When running:
```fish
journalctl --user -u aura-accent-sync -f
```
You may see a warning like:
```log
libayatana-appindicator is deprecated. Please use libayatana-appindicator-glib in newly written code.
```

This warning originates from underlying system libraries used by the tray icon implementation. It's purely cosmetic and doesn't affect functionality. The `tray-icon` crate maintainers are aware of this and will address it in future updates.

**Missing `libxdo.so.3` Error**

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
   ```bash
   sudo apt-get install libxdo-dev
   ```
   ```fish
   paru -S xdotool  # On Arch/CachyOS
   ```
   or
   ```sh
   sudo pacman -S xdotool  # On Arch/CachyOS
   ```
This issue occurs because the binary was compiled against a specific version of system libraries that may have been updated by your package manager.

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
Contributions are welcome! Please:

1. Fork the repository.
2. Create a feature branch (`git checkout -b my-feature`).
3. Submit a pull request with a concise description of the change.
4. Ensure code follows Rust idioms and passes `cargo fmt` & `cargo clippy`.

**Code of Conduct** – By participating you agree to follow the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/version/3/0/code_of_conduct/).

---

*Enjoy a perfectly synced glow between your Asus hardware and your Desktop Environment!*
