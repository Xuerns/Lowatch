# PRD — Lowatch (Desktop App)

| | |
|---|---|
| **App name** | Lowatch |
| **Document version** | 1.1 |
| **Date** | November 16, 2026 |
| **Status** | Implemented MVP |
| **Target platform** | Windows 10/11 (primary), macOS & Linux X11 (best-effort) |
| **License** | Apache-2.0 |

---

## 1. Overview

**Lowatch** is a lightweight desktop application for running **multiple stopwatches simultaneously**, where each stopwatch has its **own name**. Users can start, pause, reset, and rename each stopwatch independently in a responsive grid layout. It also includes **Focus Mode**: the application window shrinks into a compact widget pinned to the **top-right corner of the screen** and stays always on top of other windows, keeping time visible without interrupting your work.

Most importantly: **stopwatch time must never be lost**. Paused stopwatches preserve their elapsed time across application restarts.

### Design Principles

1. **Simple** — a single database table, no ORM, no external state management library, no backend server.
2. **Rust is the source of truth** — all timing logic and persistence live in Rust; the frontend is purely presentational.
3. **Accurate via timestamps, not tick counters** — elapsed time is computed from timestamp deltas rather than incrementing a `setInterval` counter, so it remains accurate even when the window is minimized or ticks are throttled.

---

## 2. Goals and Non-Goals

### Goals (MVP)

- Create, name, rename, and delete multiple stopwatches.
- Start/resume, pause, and reset each stopwatch independently.
- **Focus Mode**: compact always-on-top widget in the top-right corner of the screen.
- **Persistent data**: paused stopwatches **do not reset** when the application is closed.
- Small footprint, fast startup, and low resource usage.
- Clean frameless desktop UI with an integrated custom titlebar (`decorations: false`), unified `#202020` background, and responsive grid layout.

### Non-Goals (intentionally excluded from MVP)

- Lap / split times.
- Countdown timer.
- Session history, statistics, or reports.
- Cloud sync / user accounts.
- Export (CSV/JSON).
- System tray, notifications, global hotkeys.
- Drag & drop reordering of stopwatches.
- Multi-language localization (UI is in English).
- Auto-updater.

---

## 3. Users and Use Cases

**Primary users:** individuals who need to track the duration of multiple parallel activities, such as students, freelancers, or developers.

| # | Use Case |
|---|---|
| UC-1 | A freelancer runs "Client A" and "Client B" stopwatches alternately to track billable hours. |
| UC-2 | A developer measures time spent on "Coding", "Meeting", and "Review" in a single day. |
| UC-3 | A user closes the app midday and reopens it the next day; all paused times are still intact. |
| UC-4 | A user is writing a document and wants one stopwatch visible as a small widget in the top-right corner of the screen. |

---

## 4. Functional Requirements

Priority: **P0** = required for MVP, **P1** = recommended for MVP, **P2** = future.

| ID | Requirement | Priority |
|---|---|---|
| F-01 | Users can create a new stopwatch. If the name is empty, it defaults to `"Stopwatch N"`. | P0 |
| F-02 | Each stopwatch has an editable name (inline edit). Names are trimmed, max 50 characters, and duplicates are allowed. | P0 |
| F-03 | **Start/Resume** button starts or resumes the stopwatch. | P0 |
| F-04 | **Pause** button pauses the stopwatch and saves accumulated time. | P0 |
| F-05 | **Reset** button resets elapsed time to `00:00:00` and stops the stopwatch. | P0 |
| F-06 | Multiple stopwatches can run simultaneously without affecting each other. | P0 |
| F-07 | Users can delete a stopwatch (with confirmation). | P0 |
| F-08 | **Focus Mode**: shrinks the window into an always-on-top widget in the top-right corner displaying one selected stopwatch. | P0 |
| F-09 | Exiting Focus Mode restores the previous window size, position, and decoration state. | P0 |
| F-10 | **Persistence:** all stopwatches (name, elapsed time, order) are stored in a local database and reloaded on startup. | P0 |
| F-11 | When closing the app, any running stopwatches are automatically paused at close time and their time is saved. | P0 |
| F-12 | Crash / power-loss recovery: stopwatches still marked running on startup are paused at their last `heartbeat_ms`. | P1 |
| F-13 | Single-instance enforcement (opening a second instance focuses the existing window). | P1 |
| F-14 | `Esc` key exits Focus Mode. | P1 |
| F-15 | Simple confirmation before Reset when elapsed time > 0. | P1 |
| F-16 | Remember normal-mode window size and position across restarts. | P2 |
| F-17 | Reorder stopwatches via drag & drop. | P2 |
| F-18 | System light/dark theme switching. | P2 |

---

## 5. Detailed Behavior

### 5.1 Stopwatch State Model

Each stopwatch stores only two timing values:

- `accumulated_ms` — total milliseconds accumulated while not running.
- `started_at_ms` — Unix timestamp (ms) when last started; `NULL` when not running.

```
elapsed(now) = accumulated_ms + (started_at_ms != NULL ? now - started_at_ms : 0)
```

Status is derived, not stored:

| Status | Condition | Primary Button Label |
|---|---|---|
| **Idle** | `started_at_ms` is `NULL` and `accumulated_ms == 0` | Start |
| **Running** | `started_at_ms` is not `NULL` | Pause |
| **Paused** | `started_at_ms` is `NULL` and `accumulated_ms > 0` | Resume |

### 5.2 State Transitions

| Action | From Idle / Paused | From Running |
|---|---|---|
| **Start / Resume** | `started_at_ms = now` | No-op (idempotent) |
| **Pause** | No-op | `accumulated_ms += now - started_at_ms`, `started_at_ms = NULL` |
| **Reset** | `accumulated_ms = 0` | `accumulated_ms = 0`, `started_at_ms = NULL` |

Note: `now` is captured **once** in Rust per action and reused for all calculations within that action.

### 5.3 Application Close and Startup

- **Normal close:** on `CloseRequested` (and `RunEvent::Exit` as a safeguard, e.g., Cmd+Q on macOS), Rust pauses all running stopwatches in a **single transaction**, then exits.
- **Startup:** all stopwatches are loaded from the database. Previously paused stopwatches remain paused with the exact same elapsed time. Nothing auto-starts.
- **Crash / power loss (F-12):** while any stopwatch is running, Rust updates `heartbeat_ms` every ±15 seconds. On startup, any row with non-NULL `started_at_ms` is recovered as `accumulated_ms += (heartbeat_ms - started_at_ms)` and `started_at_ms = NULL`. Maximum time loss is ±15 seconds, and offline time is never counted.

> **Product decision:** stopwatches running when the app closes **do not keep counting** while the app is closed. This is safer and matches typical desktop stopwatch expectations. See Section 13 for alternatives.

#### 5.3.1 Close = Quit

In MVP, closing the window quits the application (no system tray). This also applies while in Focus Mode.

### 5.4 Focus Mode

**Enter (`enter_focus_mode(id)`), handled entirely in Rust:**

1. Save current window geometry in memory (size, position, maximized state, decorations, resizable).
2. If maximized, unmaximize first.
3. Disable window decorations and resizing, and enable **always-on-top**.
4. Resize the window to **280 × 96 px (logical)**.
5. Position the window at the **top-right corner** of the current monitor's work area with a 16 px margin. On multi-monitor setups, use the current monitor rather than the primary monitor.
6. Switch the frontend to the compact view.

**Focus Mode UI:** stopwatch name, `HH:MM:SS` time display, Pause/Resume button, and exit button (`✕`). The entire widget is draggable (`data-tauri-drag-region`).

**Exit (`exit_focus_mode()`):** restore all saved window properties (`decorations`, `resizable`, `always-on-top = false`, size, position, maximized state), then return the frontend to normal mode.

**Additional rules:**

- Running stopwatches continue running when entering or exiting Focus Mode.
- Focus Mode is **not persisted**: on restart, the app always opens in normal mode.
- Closing the window in Focus Mode behaves the same as Section 5.3.
- Reset and Delete are unavailable in Focus Mode to prevent accidental clicks on the compact widget.

### 5.5 Validation and Constraints

- Name: trimmed, 1–50 characters, no control characters. If empty on create, generate a default name; if invalid on rename, reject with an error.
- No hard limit on the number of stopwatches; the grid container scrolls vertically.
- Display format: `HH:MM:SS.t` (with tenths of a second) in normal mode. Hours expand naturally beyond 99 hours without truncation. Focus Mode omits tenths of a second (`HH:MM:SS`).

---

## 6. Tech Stack

| Layer | Choice | Notes |
|---|---|---|
| Desktop shell | **Tauri 2** | Small binary using the OS native WebView. |
| Backend | **Rust** (full) | All timing logic, SQLite persistence, and Focus Mode window manipulation. |
| Frontend | **SvelteKit** + Svelte 5 + TypeScript | SPA mode: `@sveltejs/adapter-static`, `ssr = false`. |
| Styling | Plain CSS (Svelte scoped styles) | No external UI library or Tailwind to keep bundle size minimal. |
| Frontend state | Svelte 5 runes (`$state`, `$derived`) | In `src/lib/stores/stopwatches.svelte.ts`. |
| Database | **SQLite via `rusqlite`** (`bundled` feature) | See Section 6.1. |
| Error handling | `thiserror` | Errors serialize to readable English strings for the frontend. |
| Tauri plugins | `tauri-plugin-single-instance` (P1) | Prevents multiple processes from writing to the same DB. |

### 6.1 Why SQLite via `rusqlite`

The dataset is small (tens of rows, one table), but **must be crash-safe**:

| Option | Assessment |
|---|---|
| **SQLite + `rusqlite`** ✅ | Single file, atomic crash-safe writes, simple synchronous API (no extra async runtime needed), near-zero configuration. |
| JSON file | Simpler initially, but requires manual atomic writes (write temp + rename) to avoid corruption on crash. |
| `sqlx` | Overkill: async runtime, compile-time query checking, and migration tooling are unnecessary. |
| ORM (Diesel / SeaORM) | Over-engineering for a single table. |
| `tauri-plugin-sql` | Exposes SQL to JavaScript, violating the "Rust is source of truth" principle. |

**Migrations** use `PRAGMA user_version`:

```rust
let v: i32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
if v < 1 {
    conn.execute_batch("CREATE TABLE ...; PRAGMA user_version = 1;")?;
}
```

**Connection settings:** `journal_mode = WAL`, `foreign_keys = ON`. A single connection wrapped in `Mutex<Connection>` is managed in Tauri `AppState`.

**File location:** Tauri `app_data_dir()`, filename `multi-stopwatch.db`.

---

## 7. Architecture

```
┌─────────────────────── Tauri App ───────────────────────┐
│                                                         │
│  SvelteKit (WebView)            Rust (src-tauri)        │
│  ┌──────────────────┐  invoke  ┌────────────────────┐   │
│  │ UI + 100ms ticker│ ───────► │ commands.rs        │   │
│  │ compute elapsed  │ ◄─────── │ (validation, logic)│   │
│  │ from snapshot    │   DTO    ├────────────────────┤   │
│  └──────────────────┘          │ db.rs (rusqlite)   │──►│ multi-stopwatch.db
│                                │ focus.rs (window)  │   │
│                                │ heartbeat thread   │   │
│                                └────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### Data Flow

1. On startup, the frontend invokes `list_stopwatches` and populates the store.
2. Each action (start/pause/reset/rename/delete) is **one IPC command**. Rust validates, writes to SQLite, and returns the updated DTO.
3. Running seconds are **not written to the database**. The frontend runs **one global ticker** (~100 ms, active only while at least one stopwatch is running) computing `elapsed = accumulated_ms + (Date.now() - started_at_ms)`.
4. Database writes occur only on user actions, app exit, and heartbeat ticks (±15 seconds while running).

Because elapsed time is derived from timestamps, minimizing the window or WebView throttling **never causes time drift**.

### Project Structure

```
Lowatch/
├─ assets/
│  └─ Logo.png                   # Source application logo
├─ docs/
│  └─ PRD.md
├─ src/                          # SvelteKit frontend
│  ├─ routes/
│  │  ├─ +layout.ts              # export const ssr = false; prerender = false
│  │  ├─ +layout.svelte          # Global styles (#202020 background)
│  │  └─ +page.svelte            # Custom titlebar, toolbar, grid view, focus switch
│  └─ lib/
│     ├─ api.ts                  # Thin invoke() wrappers
│     ├─ time.ts                 # computeElapsedMs(), formatElapsed()
│     ├─ stores/
│     │  └─ stopwatches.svelte.ts
│     └─ components/
│        ├─ StopwatchCard.svelte
│        └─ FocusView.svelte
├─ static/
│  ├─ logo.png                   # Titlebar brand icon
│  └─ favicon.png
├─ src-tauri/
│  ├─ src/
│  │  ├─ main.rs
│  │  ├─ lib.rs                  # Builder, setup, heartbeat, window/run events
│  │  ├─ models.rs               # Stopwatch, StopwatchDto
│  │  ├─ stopwatch.rs            # Pure timing logic (start/pause/reset/elapsed)
│  │  ├─ db.rs                   # Open, migrate, query, crash recovery
│  │  ├─ commands.rs             # Tauri IPC commands
│  │  ├─ focus.rs                # Enter/exit Focus Mode
│  │  └─ error.rs                # AppError enum
│  ├─ capabilities/default.json  # Minimal permissions + custom titlebar controls
│  ├─ icons/                     # Generated from assets/Logo.png
│  ├─ tauri.conf.json
│  └─ Cargo.toml
├─ package.json
└─ svelte.config.js              # adapter-static, fallback: 'index.html'
```

---

## 8. Data Model

```sql
CREATE TABLE stopwatches (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    name           TEXT    NOT NULL,
    accumulated_ms INTEGER NOT NULL DEFAULT 0,   -- accumulated time while not running
    started_at_ms  INTEGER,                      -- unix ms; NULL = not running
    heartbeat_ms   INTEGER,                      -- last seen alive (for crash recovery)
    position       INTEGER NOT NULL,             -- display order
    created_at_ms  INTEGER NOT NULL,
    updated_at_ms  INTEGER NOT NULL
);
```

### Core Logic (pure functions for testability)

```rust
pub fn elapsed_ms(sw: &Stopwatch, now_ms: i64) -> i64 {
    match sw.started_at_ms {
        Some(t) => sw.accumulated_ms + (now_ms - t).max(0),
        None => sw.accumulated_ms,
    }
}

pub fn start(sw: &mut Stopwatch, now_ms: i64) {
    if sw.started_at_ms.is_none() {
        sw.started_at_ms = Some(now_ms);
    }
}

pub fn pause(sw: &mut Stopwatch, now_ms: i64) {
    if let Some(t) = sw.started_at_ms.take() {
        sw.accumulated_ms += (now_ms - t).max(0);
    }
}

pub fn reset(sw: &mut Stopwatch) {
    sw.accumulated_ms = 0;
    sw.started_at_ms = None;
}
```

### Frontend DTO

```ts
type StopwatchDto = {
  id: number;
  name: string;
  accumulated_ms: number;
  started_at_ms: number | null;
  status: "idle" | "running" | "paused";
  position: number;
};
```

---

## 9. IPC Command Contract

All commands return `Result<T, String>`. Errors are displayed in the UI error banner.

| Command | Arguments | Return | Notes |
|---|---|---|---|
| `list_stopwatches` | — | `StopwatchDto[]` | Ordered by `position`. |
| `create_stopwatch` | `name?: string` | `StopwatchDto` | Defaults to `"Stopwatch N"`. |
| `rename_stopwatch` | `id, name` | `StopwatchDto` | Validated 1–50 characters. |
| `start_stopwatch` | `id` | `StopwatchDto` | Idempotent. |
| `pause_stopwatch` | `id` | `StopwatchDto` | Idempotent. |
| `reset_stopwatch` | `id` | `StopwatchDto` | Stops and clears elapsed time. |
| `delete_stopwatch` | `id` | `()` | |
| `enter_focus_mode` | `id` | `()` | Resizes & pins window (see 5.4). |
| `exit_focus_mode` | — | `()` | Restores window geometry. |

Rust lifecycle hooks (non-command):

- `on_window_event(CloseRequested)` and `RunEvent::Exit` call `pause_all_on_exit()`.
- Heartbeat thread updates `heartbeat_ms` every ±15 seconds while any stopwatch is running.
- Startup runs DB migrations, then `recover_crashed_running()` (F-12).

---

## 10. UI Specification

### Main Window

Initial size `480 × 640 px`, minimum size `360 × 400 px`, `decorations: false`.

```
┌──────────────────────────────────────────────┐
│  [Logo] Lowatch                     ─  □  ✕  │
│  [ Stopwatch name (optional)... ]  [ + New ] │
├──────────────────────────────────────────────┤
│ ┌───────────────────┐  ┌───────────────────┐ │
│ │ READY             │  │ RUNNING           │ │
│ │ Study Rust     ✎  │  │ Meeting        ✎  │ │
│ │    01:23:45.6     │  │    00:12:03.0     │ │
│ │ [Pause][Rst][Fcs] │  │ [Resume][Rst][Fcs]│ │
│ └───────────────────┘  └───────────────────┘ │
└──────────────────────────────────────────────┘
```

- **Header & Theme:** Uses a custom integrated titlebar (`decorations: false`) with the Lowatch logo and single app title aligned with Minimize, Maximize/Restore, and Close buttons. Header and body share the same `#202020` background color (inspired by Windows Clock).
- **Grid Layout:** Stopwatches are displayed in a responsive CSS Grid (`grid-template-columns: repeat(auto-fill, minmax(210px, 1fr))`).
- **Empty state:** Displays `"No stopwatches yet"` with a `"+ Create stopwatch"` button.
- **Status indicators:** Running, paused, and ready states are visually distinguished (`RUNNING`, `PAUSED`, `READY`).
- **Inline rename:** Click `✎` or double-click the stopwatch name; `Enter` saves, `Esc` cancels.
- **Tabular numbers:** Time displays use `font-variant-numeric: tabular-nums` to prevent horizontal jitter.
- **Confirmations:** Delete and Reset (when elapsed time > 0) prompt an inline confirmation.

### Focus Mode

```
┌──────────────────────────────┐
│ Study Rust              [✕]  │   ← entire widget is draggable
│ 01:23:45           [ Pause ] │
└──────────────────────────────┘
        280 × 96 px, top-right of screen
```

- Frameless, always-on-top (`decorations: false`).
- `[✕]` (or `Esc`) exits Focus Mode without closing the app.

---

## 11. Non-Functional Requirements

| Area | Target |
|---|---|
| **Accuracy** | Elapsed time depends solely on system timestamps; no cumulative UI ticker drift. |
| **Data durability** | Zero data loss on normal close. On crash, max ±15 seconds loss for running stopwatches; paused time is never lost. |
| **Performance** | Startup < 1s on typical hardware. Near-0% idle CPU when no stopwatch is running. |
| **Size** | Installer in the few-MB range (typical for Tauri). |
| **Privacy** | 100% local data. No network requests, telemetry, or accounts. |
| **Security** | Minimal Tauri capabilities (`core:default` + custom titlebar window controls). No filesystem/shell access from frontend. Parameterized SQL in Rust only. |
| **Accessibility** | Descriptive labels / `aria-label` attributes, high contrast, keyboard operable (`Tab`/`Enter`/`Esc`). |

---

## 12. Edge Cases

| Case | Behavior |
|---|---|
| Rapid double-click on Start | Safe: command is idempotent; time does not reset or double-count. |
| Computer sleep/hibernate while running | Time continues based on wall clock (standard stopwatch behavior). |
| System clock adjusted while running | `(now - started_at).max(0)` prevents negative deltas. |
| Opening a second app instance | Second instance focuses the existing window (F-13). |
| Missing database file | Created automatically with schema migration. |
| Corrupted/unreadable database file | Clear error shown; original file is never silently overwritten. |
| Deleting a stopwatch while in Focus Mode | Not possible, as Delete is not exposed in Focus Mode. |
| Multi-monitor with mixed DPI scaling | Top-right position computed from current monitor's work area and scale factor. |
| App closed while in Focus Mode | Running stopwatches are paused and saved; next launch starts in normal mode. |
| Very long stopwatch name | Capped at 50 characters; truncated with ellipsis in the UI. |

---

## 13. Open Questions and Assumptions

| # | Question | Default Used |
|---|---|---|
| 1 | Primary target OS? | Windows 10/11. macOS and Linux (X11) compile, with best-effort Focus Mode support. |
| 2 | Running stopwatches on close: keep counting or pause? | **Pause** on close. Keeping wall-clock counting across restarts can be added as a setting in a future version. |
| 3 | System tray to keep app alive on close? | No for MVP. Close = quit. |
| 4 | Multiple stopwatches in Focus Mode? | No. MVP shows one stopwatch; switching (prev/next) is a P2 candidate. |
| 5 | Final app name and icon? | `"Lowatch"` and icon generated from `assets/Logo.png`. |

---

## 14. Test Plan

### Unit & Integration Tests (Rust)

- `stopwatch.rs`: elapsed in idle/running/paused, multi-cycle start/pause accumulation, idempotent double start/pause, reset from running/paused, negative clock-skew guard, crash recovery via `heartbeat_ms`.
- `db.rs`: `PRAGMA user_version` migration, CRUD lifecycle, default numbering after deletions, start/pause/reset persistence, `pause_all_running` transaction, corrupted DB preservation, persistence across reopen.
- `commands.rs`: full CRUD and start/pause/reset command flows.
- `focus.rs`: top-right position math across 100%/125%/150%/200% DPI, multi-monitor positive/negative offsets, invalid scale fallback, geometry preservation.
- `lib.rs`: idempotent `pause_all_on_exit`, heartbeat tick + startup crash recovery flow.

### Manual Verification Checklist

- [ ] Create 3 stopwatches, run 2 simultaneously; verify independent timing.
- [ ] Pause stopwatch A, close app, reopen: A remains paused with the exact same time.
- [ ] Leave stopwatch B running, close app: B is saved in paused state.
- [ ] Kill process via Task Manager while running, reopen: time recovers within ≤ 15 seconds.
- [ ] Enter Focus Mode: widget appears at top-right, stays always-on-top, and is draggable.
- [ ] Exit Focus Mode: normal window size and position are restored.
- [ ] Test Focus Mode on multi-monitor setups with different DPI scaling.
- [ ] Minimize window for 10 minutes while running: time remains accurate upon restore.
- [ ] Launch app twice: only one window opens.

---

## 15. MVP Acceptance Criteria

MVP is complete when:

1. All **P0** requirements are implemented.
2. *"Pause → close app → reopen"* always preserves exact elapsed time.
3. Focus Mode pins the widget to the top-right corner, always-on-top, and restores normal geometry on exit.
4. All Rust unit and integration tests pass.
5. Windows installer builds cleanly (`pnpm tauri build`) and runs on a clean machine.

---

## 16. Milestones

| Milestone | Scope | Outcome |
|---|---|---|
| **M0 — Scaffold** | Tauri 2 + SvelteKit + TS, `adapter-static`, `ssr = false`. | Empty app runs in `tauri dev`. |
| **M1 — Core Rust** | `stopwatch.rs` + tests, `db.rs` + migrations, IPC commands. | Core logic tested without UI. |
| **M2 — Main UI** | Store, global ticker, `StopwatchCard`, grid layout, inline rename, confirmations. | Core features usable in UI. |
| **M3 — Persistence** | Close/exit hooks, heartbeat thread, crash recovery, single-instance. | Time safe across close and crash. |
| **M4 — Focus Mode** | `focus.rs`, `FocusView`, drag region, `Esc`, geometry restore. | Top-right widget functional. |
| **M5 — Polish & Release** | Lowatch branding & icon, custom frameless titlebar, English UI, build. | Release-ready app. |

---

## 17. Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Linux Wayland lacks reliable `set_position` / always-on-top | Focus Mode imprecise on Wayland | Windows is primary target; document X11 recommendation on Linux. |
| Window-state plugin (if added in P2) conflicts with Focus Mode | Widget size saved as normal size | Disable state saving during Focus Mode or manage geometry manually. |
| Window operation ordering differs across OSes | Wrong widget size/position | Encapsulate in `focus.rs` and test on target OSes. |
| System clock changes affect running stopwatches | Incorrect delta | Clamp negative values; optionally switch to monotonic `Instant` for active sessions later. |
| WebView throttled while hidden | Delayed visual refresh | No impact on accuracy (timestamp-based); UI catches up on next tick. |
| Over-engineering | Scope creep | Stick to Non-Goals; new features require a PRD revision. |

---

## 18. Core Dependencies

**Rust (`src-tauri/Cargo.toml`)**

```toml
tauri = { version = "2" }
tauri-plugin-single-instance = "2"
rusqlite = { version = "0.32", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
```

**Frontend (`package.json`)**

```
svelte (^5), @sveltejs/kit (^2), @sveltejs/adapter-static (^3),
vite (^8), typescript (~6), @tauri-apps/api (^2), @tauri-apps/cli (^2)
```

**`src-tauri/tauri.conf.json` highlights**

- `productName`: `"Lowatch"`, `identifier`: `"com.xuerns.lowatch"`
- `build.frontendDist = "../build"`, `build.devUrl = "http://localhost:1420"`
- `build.beforeDevCommand = "npm run dev"`, `build.beforeBuildCommand = "npm run build"`
- Main window: `480 × 640`, `minWidth: 360`, `minHeight: 400`, `decorations: false`
