# ADPOWER (WinRS-DCX Python Version)

This is the Python-based regeneration of the WinRS-DCX project, renamed to **ADPOWER** and redesigned with a professional native GUI inspired by the original ADPOWER software.

## Features
- **Native GUI:** Built with PySide6 (Qt) for a high-performance, professional desktop experience.
- **ALLDSP Style:** Dark theme, vertical channel strips, and streamlined navigation.
- **Direct Serial Control:** Communicates with Behringer DCX2496 via RS232 using `pyserial`.
- **Asynchronous Processing:** Background polling and state synchronization ensure a responsive UI.

## Requirements
- Python 3.9+
- Dependencies listed in `requirements.txt`

## Getting Started

1.  Navigate to the `python_app` directory:
    ```bash
    cd python_app
    ```
2.  Install the required dependencies:
    ```bash
    pip install -r requirements.txt
    ```
3.  Run the application:
    ```bash
    python main.py
    ```

## Project Structure
- `main.py`: Entry point.
- `src/protocol/`: Ported Rust logic for DCX2496 SysEx communication and parameter mappings.
- `src/ui/`: PySide6 components and custom widgets.
- `src/utils/`: Background workers and serial management.

## Porting Notes
## Building the Executable

To generate a standalone Windows `.exe` file:

1.  Run the build script from the project root:
    ```bash
    scripts\build_python.bat
    ```
2.  The resulting executable will be located in `python_app/dist/ADPOWER.exe`.

This build uses `PyInstaller` to bundle the Python interpreter, dependencies, and resources into a single file.
