# WinRS-DCX (DuinoDCX Management)

WinRS-DCX is a management application for the [Behringer Ultradrive Pro / DCX2496](https://www.behringer.com/p/P0B6H) audio processor. Originally an ESP32-based Wi-Fi controller, this project has evolved into a native application built with Rust that supports both **Windows** and **OpenWRT (iStoreOS)**.

The application embeds the [dcx-ui](https://github.com/lasselukkari/dcx-ui) and communicates directly with the DCX2496 hardware over an RS232-to-USB serial connection.

<img src="https://i.imgur.com/2BB5tPi.png" width="100%" />

## Disclaimer
I take no responsibility if you destroy your sound system using this.

## Requirements
* **Hardware:** RS232 to USB Converter cable connected to the DCX2496.
* **Windows:** Windows 10/11 (for the `.exe` version).
* **OpenWRT:** A router running OpenWRT (like iStoreOS) with USB-Serial support (typically `kmod-usb-serial-ch341` or `kmod-usb-serial-pl2303`).

## Platform Support

### Windows
The easiest way to run the application on Windows:
1. Download or build the `WinRS-DCX-Management.exe`.
2. Run the executable.
3. Open your browser to `http://localhost:3000`.

### OpenWRT / iStoreOS (ARM & x86)
You can run WinRS-DCX directly on your router.

#### 1. Identification
Run `uname -m` on your router to find your architecture:
*   `aarch64`: 64-bit ARM (Common in modern routers like RK3568/iStoreOS).
*   `armv7l`: 32-bit ARM.
*   `x86_64`: Intel/AMD based routers.

#### 2. Build / Cross-Compilation
Use Docker to build for your specific architecture:
```bash
chmod +x scripts/cross-build.sh
./scripts/cross-build.sh
```
Binaries will be generated in `duinodcx-rs/target/`.

#### 3. Installation
Copy the binary to `/usr/bin/duinodcx` on your router and ensure it has execute permissions. 

**Serial Port Permissions:**
On OpenWRT, you may need to ensure your user has access to the serial device (e.g., `/dev/ttyUSB0`). By default, the service script runs as root.

#### 4. Service Management
A `Makefile` and init script are provided in the `openwrt/` directory for integration into the OpenWRT build system or for manual installation.

## Building from Source

Ensure you have [Rust and Cargo](https://rustup.rs/) installed.

### Windows Build
Run `scripts\build.cmd`. This will compile the Rust application and produce `WinRS-DCX-Management.exe` in the root.

### UI Development
The UI is located in `dcx-ui-old`. If you modify it, run `npm run build` inside that folder to refresh the embedded assets.

## Acknowledgements
* **Lasse Lukkari** - Creator of the original DuinoDCX and dcx-ui.
* **Ilkka Huhtakallio** - For contributing the transfer function code for the user interface.
* **DCX Link** - For the original introduction video.

## License
[MIT](LICENSE)
