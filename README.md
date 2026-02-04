# aura-plasma-sync

A lightweight KDE‑native system tray utility that synchronizes **Asus Aura** lighting with the current **Plasma accent color**.  
It listens for **DBus signals** from the XDG Settings Portal for instantaneous updates, with a fallback to `kdeglobals` via `kreadconfig6`.

---

## Table of Contents
- [Features](#features)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
  - [Manual Build & Install](#manual-build--install)
  - [Systemd Service (Recommended)](#systemd-service-recommended)
- [Usage](#usage)
- [Configuration & Customization](#configuration--customization)
- [Development](#development)
  - [Running Checks](#running-checks)
  - [Building](#building)
  - [Testing](#testing)
- [License](#license)
- [Contributing](#contributing)

---

## Features
- **Real‑time DBus Sync**: Listen for `SettingChanged` signals from `org.freedesktop.portal.Settings`. Updates are instantaneous with zero filesystem polling overhead.
- **Fast-Path Extraction**: Directly parses RGB double-precision values from the DBus payload, avoiding external process spawns for most updates.
- **Robust Fallbacks**: 
  1. Primary: Direct DBus signal payload for `accent-color`.
  2. Secondary: `General` → `AccentColor` from `kdeglobals` (via `kreadconfig6`).
  3. Tertiary: `Colors:Selection` → `BackgroundNormal` from `kdeglobals` (via `kreadconfig6`).
- **Systemd user service**: Includes an installer to run the utility as a persistent background service.
- **Tray integration**: Provides a native Plasma system tray icon with a toggle to enable/disable synchronization.

---

## Prerequisites

- **KDE Plasma 6** (ensure `kreadconfig6` is installed for fallback functionality).
- **xdg-desktop-portal-kde** (standard in Plasma 6 for DBus signals).
- [**asusctl**](https://gitlab.com/asus-linux/asusctl) – the utility that controls Asus Aura devices.
  The project has been tested with **`asusctl v6.3.2`**.

  The required syntax is:  
```bash
asusctl aura effect static -c <hex-color>
```

- Optional but recommended: `systemd` (user‑level) for running the service.

Utility dependencies (Rust crates):
- `zbus` – DBus communication (XDG Settings Portal).
- `anyhow` – Flexible error handling.
- `clap` – Command‑line argument parsing.
- `ksni` – Native KDE System Tray integration.
- `which` – Binary existence checks.
- `log` / `env_logger` – Standard logging.

---

## Installation

### Manual Build & Install
```bash
git clone https://github.com/neilpandya/aura-plasma-sync
cd aura-plasma-sync
cargo build --release
# The binary will be at target/release/aura-plasma-sync
sudo cp target/release/aura-plasma-sync /usr/local/bin/
```

### Systemd Service (Recommended)
The project ships an installer that creates a **user‑level systemd service** pointing to the binary you just built.

```bash
cargo install --path .
aura-plasma-sync install
```

The installer:
1. Detects the current executable path.
2. Writes a unit file into `~/.config/systemd/user/aura-plasma-sync.service`.
3. Reloads the systemd daemon.
4. Enables and starts the service immediately.

To verify it’s running:
```bash
systemctl --user status aura-plasma-sync.service
```

To stop or disable later:
```bash
aura-plasma-sync uninstall
```

---

## Usage
When the service is running, a tray icon appears in the Plasma panel.

- **Active** (green check‑mark) – the sync thread is alive and listening for changes.
- **Inactive** (gray icon) – syncing is paused; you can toggle it via the tray menu entry **“Sync Enabled” / “Sync Disabled”**.

The utility works in the background; you do not need to interact with it directly unless you wish to change the toggle.

---

## Configuration & Customization
The program does **not** require any external configuration files.  
All behavior is driven by the presence of keys in `kdeglobals`:

| Key/Group                     | Purpose |
|-------------------------------|---------|
| `General → AccentColor`       | Primary source for the accent color. |
| `Colors:Selection → BackgroundNormal` | Fallback source if `AccentColor` is absent. |

If neither key exists, the program exits with a clear error message explaining that an accent color could not be determined.

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

*Enjoy a perfectly synced glow between your Asus hardware and KDE Plasma!*
