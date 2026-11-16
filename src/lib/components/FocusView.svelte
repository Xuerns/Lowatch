<script lang="ts">
  import type { StopwatchDto } from "../api";
  import { computeElapsedMs, formatElapsed } from "../time";

  interface Props {
    stopwatch: StopwatchDto;
    nowMs: number;
    onStart: (id: number) => Promise<void> | void;
    onPause: (id: number) => Promise<void> | void;
    onExit: () => Promise<void> | void;
  }

  let { stopwatch, nowMs, onStart, onPause, onExit }: Props = $props();

  let elapsedMs = $derived(computeElapsedMs(stopwatch, nowMs));
  let formattedTime = $derived(formatElapsed(elapsedMs, false));

  let primaryActionLabel = $derived(
    stopwatch.status === "running"
      ? "Pause"
      : stopwatch.status === "paused"
        ? "Resume"
        : "Start"
  );

  async function handlePrimaryAction(): Promise<void> {
    if (stopwatch.status === "running") {
      await onPause(stopwatch.id);
    } else {
      await onStart(stopwatch.id);
    }
  }

  function handleWindowKeydown(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      event.preventDefault();
      void onExit();
    }
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<section
  class="focus-widget"
  data-tauri-drag-region
  aria-label="Mode fokus stopwatch {stopwatch.name}"
>
  <div class="focus-top" data-tauri-drag-region>
    <span class="focus-name" data-tauri-drag-region title={stopwatch.name}>
      {stopwatch.name}
    </span>
    <button
      type="button"
      class="btn-exit"
      onclick={() => onExit()}
      aria-label="Keluar dari mode fokus (Esc)"
      title="Keluar dari mode fokus (Esc)"
    >
      ✕
    </button>
  </div>

  <div class="focus-bottom" data-tauri-drag-region>
    <div
      class="focus-time"
      data-status={stopwatch.status}
      data-tauri-drag-region
    >
      {formattedTime.formatted}
    </div>
    <button
      type="button"
      class="btn-action"
      data-status={stopwatch.status}
      onclick={handlePrimaryAction}
    >
      {primaryActionLabel}
    </button>
  </div>
</section>

<style>
  .focus-widget {
    width: 100vw;
    height: 100vh;
    box-sizing: border-box;
    overflow: hidden;
    background-color: #111111;
    border: 1px solid #262626;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    user-select: none;
    cursor: grab;
  }

  .focus-widget:active {
    cursor: grabbing;
  }

  .focus-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    min-width: 0;
  }

  .focus-name {
    flex: 1;
    min-width: 0;
    font-size: 0.8125rem;
    font-weight: 600;
    color: #a3a3a3;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.25;
  }

  .btn-exit {
    background-color: transparent;
    color: #737373;
    border: 1px solid transparent;
    border-radius: 6px;
    width: 24px;
    height: 24px;
    padding: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-family: inherit;
    font-size: 0.8125rem;
    line-height: 1;
    cursor: pointer;
    flex-shrink: 0;
    transition:
      background-color 120ms ease,
      color 120ms ease,
      border-color 120ms ease;
  }

  .btn-exit:hover {
    background-color: #171717;
    color: #f5f5f5;
    border-color: #262626;
  }

  .focus-bottom {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    min-width: 0;
  }

  .focus-time {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-variant-numeric: tabular-nums;
    font-feature-settings: "tnum" 1;
    font-size: 1.625rem;
    font-weight: 700;
    line-height: 1.1;
    letter-spacing: -0.02em;
    color: #a3a3a3;
  }

  .focus-time[data-status="running"] {
    color: #ffffff;
  }

  .focus-time[data-status="paused"] {
    color: #e5e5e5;
  }

  .btn-action {
    min-width: 78px;
    height: 34px;
    padding: 0 12px;
    background-color: #f5f5f5;
    color: #0a0a0a;
    border: 1px solid #f5f5f5;
    border-radius: 6px;
    font-family: inherit;
    font-size: 0.8125rem;
    font-weight: 600;
    cursor: pointer;
    flex-shrink: 0;
    transition:
      background-color 120ms ease,
      color 120ms ease,
      border-color 120ms ease;
  }

  .btn-action:hover {
    background-color: #e5e5e5;
    border-color: #e5e5e5;
  }

  .btn-action[data-status="running"] {
    background-color: #171717;
    color: #f5f5f5;
    border: 1px solid #404040;
  }

  .btn-action[data-status="running"]:hover {
    background-color: #1f1f1f;
    border-color: #525252;
  }
</style>
