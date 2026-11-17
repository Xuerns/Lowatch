<div align="center">
  <img src="assets/Logo.png" alt="Lowatch Logo" width="120" />
  <h1>Lowatch</h1>
  <p>Lightweight, open-source multi-stopwatch desktop app for Windows</p>

  <p>
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.85+-black?logo=rust&logoColor=white" alt="Rust Version"></a>
    <a href="https://v2.tauri.app/"><img src="https://img.shields.io/badge/Tauri-v2-24C8DB?logo=tauri&logoColor=white" alt="Tauri v2"></a>
    <a href="https://kit.svelte.dev/"><img src="https://img.shields.io/badge/SvelteKit-5-FF3E00?logo=svelte&logoColor=white" alt="SvelteKit 5"></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache--2.0-blue.svg" alt="License: Apache-2.0"></a>
  </p>
</div>

## Overview

Lowatch is a local-first desktop app for tracking multiple tasks at once. Each stopwatch has its own name and sits in a grid where you can start, pause, reset, or rename it independently.

For single-task work, Focus Mode shrinks the window into a compact, always-on-top widget pinned to the top-right corner of the screen. The app calculates elapsed time from system timestamps instead of UI tick counters, so tracking stays accurate even when minimized. All state is stored locally in SQLite without accounts or remote servers.

## License

This project is licensed under the Apache License 2.0. See the [LICENSE](LICENSE) file for details.
