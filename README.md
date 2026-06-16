# krdiffgui

A simple desktop GUI for applying KrDiff patch files.

## Features

- Select directories through a native file dialog
- Apply `.krpdiff` and `.krdiff` patch files
- Process all patch files in a selected patch directory
- Display execution logs in the UI
- Cross-platform desktop application powered by Tauri

## Requirements

Before running the project, make sure you have the following installed:

- [Node.js](https://nodejs.org/)
- npm
- [Rust](https://www.rust-lang.org/tools/install)
- Tauri system dependencies for your operating system

For platform-specific Tauri setup instructions, see the official documentation:

https://tauri.app/start/prerequisites/

## Development

Start the application in development mode:
This starts the Vite development server and launches the Tauri desktop application.

`npm install`

`npm run tauri dev`

## Build

Build the application for production:
The generated application bundles will be available under the Tauri target directory.

`npm install`

`npm run tauri build`

## Usage

1. Launch the application.
2. Select or enter the `source_dir`.
   `F:\Wuthering Waves Game`
3. Select or enter the `patch_dir`.
   `F:\Wuthering Waves Game\launcherDownload\3.5.0`
4. Select or enter the `output_dir`.
   `F:\Wuthering Waves Game`
5. Click the run button to apply patches.
6. Check the log panel for progress and results.

