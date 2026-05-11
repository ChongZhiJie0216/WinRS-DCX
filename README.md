# WinRS-DCX (DuinoDCX Management)

A Rust-based Windows management application for the [Behringer Ultradrive Pro / DCX2496](https://www.behringer.com/p/P0B6H).

This project is a native Windows adaptation of the original [DuinoDCX](https://github.com/lasselukkari/DuinoDCX), transitioning from an ESP32-based Wi-Fi controller to a local desktop application built with Rust. It embeds the [dcx-ui](https://github.com/lasselukkari/dcx-ui) and communicates directly with the DCX2496 hardware over an RS232-to-USB serial connection.

<img src="https://i.imgur.com/2BB5tPi.png" width="100%" />

## Disclaimer
I take no responsibility if you destroy your sound system using this.

## Requirements
* Windows Operating System
* RS232 to USB Converter cable connected to the DCX2496

## Getting Started

### Building from Source

Ensure you have [Rust and Cargo](https://rustup.rs/) installed.

To build the executable and set up the workspace:
1. Run the `scripts\build.cmd` script. This will compile the Rust application, clear necessary caches, and copy the executable to the root directory.
2. The compiled binary will be placed in the project root as `WinRS-DCX-Management.exe`.

### Running

Run `WinRS-DCX-Management.exe` to start the local backend server. Open your web browser to the provided localhost address to access and use the management UI.

## Acknowledgements
* **Lasse Lukkari** - Creator of the original DuinoDCX and dcx-ui.
* **Ilkka Huhtakallio** - For contributing the transfer function code for the user interface. Without him this project would not have all those pretty graphs.
* **DCX Link** - For the original introduction video.

## License
[MIT](LICENSE)