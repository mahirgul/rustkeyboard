# ⌨️ RustKeyboard — MSI Keyboard RGB LED Controller

<img width="459" height="689" alt="image" src="https://github.com/user-attachments/assets/e519e69d-c978-4a08-8945-2b3b51d0660e" />


<div align="center">

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Windows](https://img.shields.io/badge/Windows_11-0078D6?style=for-the-badge&logo=windows&logoColor=white)
![MSI](https://img.shields.io/badge/MSI-FF0000?style=for-the-badge&logo=msi&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)

**A lightweight, native RGB LED controller for MSI-based gaming laptops**
*Built with Rust + Native Windows Win32 API — no bloated software needed.*

[Features](#-features) • [Screenshots](#-screenshots) • [Installation](#-installation) • [Compatibility](#-compatibility) • [Building](#%EF%B8%8F-building-from-source) • [Hotkey](#-hotkey) • [FAQ](#-faq)

</div>

---

## 🎯 The Problem

MSI gaming laptops (and MSI-based OEM brands like **Game Garaj**) ship with keyboard RGB backlighting, but:

- ❌ **MSI Center / SteelSeries GG** is bloated, uses 300+ MB RAM, and often fails to detect the keyboard
- ❌ The default lighting software **doesn't support OEM/rebranded MSI laptops** (Game Garaj, XPG, etc.)
- ❌ Keyboards sometimes get **stuck in bootloader mode** after sleep/hibernate — LEDs stop working entirely
- ❌ No simple way to **quickly toggle lighting modes** without opening a full application

## ✅ The Solution

**RustKeyboard** is a tiny (~11 MB), zero-dependency native app that:

- 🎨 Directly controls your keyboard LEDs via **USB HID** — no MSI Center needed
- 🔧 **Automatically recovers** keyboards stuck in bootloader mode (Artery MCU)
- ⌨️ **Fn+F8 hotkey** to cycle through hardware lighting modes instantly
- 🖥️ Runs silently in the **system tray** with **0% CPU** usage when minimized
- 🌍 Supports **English**, **German (Deutsch)**, and **Turkish (Türkçe)** interfaces

---

## ✨ Features

### 🔴🟢🔵 Hardware Lighting Modes
| Mode | Description |
|------|-------------|
| Off | All LEDs disabled |
| Static Single Color | Solid color across all zones |
| Breathing Wave | Smooth fade in/out |
| Single Flashing | Single blink pattern |
| Double Flashing | Double blink pattern |
| Multi-color Shift | Colors shift across zones |
| Rainbow | Full RGB spectrum sweep |

### 🛠️ Other Features
- **7-zone color customization** — pick any color for each keyboard zone
- **System tray** — minimize to tray, runs in background with 0% CPU usage
- **Start on boot** — optional Windows auto-start via Registry
- **Bootloader recovery** — auto-detect and wake up stuck keyboards
- **Fn+F8 hotkey** — cycle through all 7 hardware modes with a single keypress
- **Multi-language UI** — English 🇬🇧, Deutsch 🇩🇪, Türkçe 🇹🇷

---

## 🖥️ Compatibility

### Developed & Tested On

| Component | Details |
|-----------|---------|
| **Laptop** | Game Garaj Slayer XL (MSI OEM) |
| **Motherboard** | MSI MS-17L2 |
| **BIOS** | E17L2IE2.103 (2021-07-22) |
| **CPU** | Intel Core i7-11800H @ 2.30GHz (11th Gen Tiger Lake) |
| **GPU** | NVIDIA GeForce RTX 3050 Ti Laptop + Intel UHD Graphics |
| **RAM** | 40 GB DDR4 |
| **OS** | Windows 11 Pro |
| **Keyboard MCU** | Artery AT32 (VID: `0x2E3C`, Bootloader PID: `0xAF01`) |

### Compatible MSI Keyboard Models

This tool communicates with MSI keyboards via **USB HID** using:
- **Vendor ID:** `0x1462` (Micro-Star International)
- **Product IDs:** `5474`, `5475`, `5476`

It should work with any MSI laptop keyboard that uses these HID identifiers, including:

| Brand | Series | Notes |
|-------|--------|-------|
| **MSI** | GE, GF, GP, GS, GT series | Steelseries per-key RGB keyboards |
| **Game Garaj** | Slayer series | Turkish MSI OEM brand |
| **XPG / ADATA** | Xenia series | Some MSI-based models |

> [!NOTE]
> If your MSI laptop's keyboard uses different PIDs, you can check with a USB HID tool and open an issue to add support.

---

## 📦 Installation

### Pre-built Binary

1. Download `rustkeyboard.exe` from the [Releases](https://github.com/mahirgul/rustkeyboard/releases) page
2. Place it in `C:\rustkeyboard\` (or any folder)
3. Run `rustkeyboard.exe`
4. *(Optional)* Enable "Start on Boot" in the Options panel

### From Source

See [Building from Source](#%EF%B8%8F-building-from-source) below.

---

## ⌨️ Hotkey

| Key | Action |
|-----|--------|
| **Fn + F8** | Cycle through hardware lighting modes: Off → Static → Breathing → Single Flash → Double Flash → Multi-color → Rainbow → Off → ... |

> [!TIP]
> The hotkey works globally — even when the app is minimized to the system tray. It saves the new mode to config automatically.

---

## 🏗️ Building from Source

### Prerequisites
- [Rust](https://rustup.rs/) (2024 edition)
- Windows 10/11 (x86_64)

### Build

```bash
git clone https://github.com/mahirgul/rustkeyboard.git
cd rustkeyboard
cargo build --release
```

The compiled binary will be at `target\release\rustkeyboard.exe`.

### Deploy

```bash
# Using the included deploy script:
deploy.bat
```

This builds in release mode and copies the exe to the project root.

---

## 📁 Project Structure

```
rustkeyboard/
├── src/
│   ├── main.rs          # Entry point, registers window class and starts loops
│   ├── window.rs        # Native Win32 window (controls, custom drawing, events)
│   ├── i18n.rs          # Localization (EN/DE/TR translations)
│   ├── hid.rs           # USB HID communication with MSI keyboards
│   ├── hook.rs          # Global keyboard hook (Fn+F8 mode cycling)
│   ├── tray.rs          # Windows system tray icon + context menu
│   ├── config.rs        # JSON config persistence
│   ├── startup.rs       # Windows Registry auto-start management
│   └── globals.rs       # Cross-thread shared state (atomics/mutexes)
├── config.json          # User configuration (auto-generated)
├── icon.ico             # System tray icon
├── wresources.rc        # Windows resource file (icon embedding)
├── build.rs             # Build script (embed-resource)
├── deploy.bat           # Build & deploy helper
├── Cargo.toml           # Rust dependencies
└── README.md            # This file
```

---

## 🔧 Configuration

Settings are stored in `config.json` next to the executable:

```json
{
  "mode": 6,
  "speed": 2,
  "brightness": 10,
  "colors": [[255,255,255], [255,0,0], [255,165,0], [255,255,0], [0,255,0], [0,255,255], [255,0,255]],
  "close_to_tray": true,
  "start_in_tray": true,
  "language": "en"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `mode` | `0-6` | Hardware effect mode |
| `speed` | `0-2` | Hardware effect speed (Slow/Medium/Fast) *[config-only]* |
| `brightness` | `0-10` | LED brightness level *[config-only]* |
| `colors` | `[R,G,B]×7` | 7 RGB color zones |
| `close_to_tray` | `bool` | Minimize to tray instead of closing |
| `start_in_tray` | `bool` | Start minimized to system tray *[config-only]* |
| `language` | `"en"/"de"/"tr"` | UI language |

---

## ❓ FAQ

**Q: My keyboard LEDs stopped working after sleep/hibernate!**
> Click "⚡ Force Wake Up" in the Actions panel. The app will send a recovery command to the Artery bootloader MCU.

**Q: The app says "Not Found" for keyboard status.**
> Make sure no other software (MSI Center, SteelSeries GG) is holding the HID device open. Close them and click "🔄 Refresh".

**Q: Does this work with per-key RGB?**
> Currently this controls keyboard-wide (zone-based) lighting, not individual per-key colors. Per-key support may be added in the future.

**Q: Can I add my own MSI keyboard PID?**
> Yes! Open `src/hid.rs` and add your PID to the `MSI_PIDS` array. You can find your keyboard's PID using [USBDeview](https://www.nirsoft.net/utils/usb_devices_view.html) or Device Manager.

**Q: How does the background CPU usage remain at 0%?**
> The application is built entirely using the native Win32 API, eliminating GUI framework overhead. When minimized to the system tray, the window is hidden (`SW_HIDE`), and standard Win32 message loop processing handles the system tray events, guaranteeing 0% CPU consumption.

---

## 📋 Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `hidapi` | 2.6.2 | USB HID communication |
| `serde` + `serde_json` | 1.0 | JSON config serialization |
| `windows-sys` | 0.52.0 | Win32 API (hooks, tray, window management) |
| `rand` | 0.9 | Randomness |
| `embed-resource` | 2.5 | Build-time icon embedding |

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).

---

<div align="center">

**Made with ❤️ and 🦀 Rust**

*If this tool helped you, please ⭐ star the repo!*

</div>
