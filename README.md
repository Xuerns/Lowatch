<div align="center">
  <img src="assets/Logo.png" alt="Lowatch Logo" width="120" />
  <h1>Lowatch</h1>
  <p><strong>Modern, Lightweight, Open Source Multi-Stopwatch Desktop App for Windows</strong></p>

  <p>
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.85+-black?logo=rust&logoColor=white" alt="Rust Version"></a>
    <a href="https://v2.tauri.app/"><img src="https://img.shields.io/badge/Tauri-v2-24C8DB?logo=tauri&logoColor=white" alt="Tauri v2"></a>
    <a href="https://kit.svelte.dev/"><img src="https://img.shields.io/badge/SvelteKit-5-FF3E00?logo=svelte&logoColor=white" alt="SvelteKit 5"></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache--2.0-blue.svg" alt="License: Apache-2.0"></a>
  </p>
</div>

## Overview

**Lowatch** is a modern, lightweight, and local-first multi-stopwatch desktop application designed for tracking multiple tasks simultaneously. Each stopwatch has its own name and can be started, paused, reset, or renamed independently in a clean grid layout.

When you need to stay focused on a single task, **Focus Mode** shrinks the window into a compact, always-on-top widget pinned to the top-right corner of your screen. Time is calculated from system timestamps rather than UI tick counters—ensuring accurate tracking even when minimized—and all state is persisted locally in SQLite with **no accounts, no logins, and no servers**.

## License

This project is licensed under the Apache License 2.0 — see the [LICENSE](LICENSE) file for details.
