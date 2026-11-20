<div align="center">
  <img src="assets/Logo.png" alt="Lowatch Logo" width="120" />
  <h1>Lowatch</h1>
  <p>Lightweight, open-source stopwatch, timer, and alarm desktop app for Windows</p>

  <p>
    <a href="https://github.com/Xuerns/Lowatch/actions/workflows/ci.yml"><img src="https://github.com/Xuerns/Lowatch/actions/workflows/ci.yml/badge.svg" alt="CI Status"></a>
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.85+-black?logo=rust&logoColor=white" alt="Rust Version"></a>
    <a href="https://v2.tauri.app/"><img src="https://img.shields.io/badge/Tauri-v2-24C8DB?logo=tauri&logoColor=white" alt="Tauri v2"></a>
    <a href="https://kit.svelte.dev/"><img src="https://img.shields.io/badge/SvelteKit-5-FF3E00?logo=svelte&logoColor=white" alt="SvelteKit 5"></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache--2.0-blue.svg" alt="License: Apache-2.0"></a>
  </p>
</div>

## Overview

Lowatch is a local-first desktop time utility built with Tauri and SvelteKit. Time tracking is calculated from system timestamps rather than UI tick counters, so values stay accurate even when minimized. All data is stored locally in SQLite with no accounts or remote servers.

## Features

- **Stopwatches:** Run multiple named stopwatches independently, switch to a maximized view, or pin one as an always-on-top mini widget with Focus Mode.
- **Timers:** Create multiple countdown timers with presets or custom durations, plus custom ringtones with a built-in audio waveform trimmer.
- **Alarms:** Schedule one-shot or repeating alarms with configurable snooze duration and full-screen ring alerts.
- **Desktop Integration:** Optional close-to-system-tray behavior so alarms and timers keep running in the background, with automatic window reveal when an alert triggers.
- **Customization:** Dark, light, and system themes, 5 dial styles, and adjustable window transparency.

## Development

Requires [Node.js](https://nodejs.org/), [pnpm](https://pnpm.io/), and [Rust](https://www.rust-lang.org/) (`1.85+`).

```bash
pnpm install
pnpm tauri dev
```

To build a release installer:

```bash
pnpm tauri build
```

## License

This project is licensed under the Apache License 2.0. See the [LICENSE](LICENSE) file for details.
