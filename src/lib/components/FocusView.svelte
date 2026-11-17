<script lang="ts">
  import type { StopwatchDto } from "../api";
  import { computeElapsedMs, formatElapsed } from "../time";
  import StopwatchDial from "./StopwatchDial.svelte";

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
  aria-label="Focus mode stopwatch {stopwatch.name}"
>
  <div class="focus-top" data-tauri-drag-region>
    <span class="focus-name" data-tauri-drag-region title={stopwatch.name}>
      {stopwatch.name}
    </span>
    <button
      type="button"
      class="btn-exit"
      onclick={() => onExit()}
      aria-label="Return to full view (Esc)"
      title="Return to full view (Esc)"
    >
      <svg
        width="14"
        height="14"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <path d="M15 3h6v6" />
        <path d="M9 21H3v-6" />
        <path d="M21 3l-7 7" />
        <path d="M3 21l7-7" />
      </svg>
    </button>
  </div>

  <div class="focus-body" data-tauri-drag-region>
    <StopwatchDial status={stopwatch.status} {elapsedMs} size={188}>
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
        aria-label="{primaryActionLabel} stopwatch {stopwatch.name}"
        title={primaryActionLabel}
      >
        {#if stopwatch.status === "running"}
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <rect x="6" y="4" width="4" height="16" rx="1" />
            <rect x="14" y="4" width="4" height="16" rx="1" />
          </svg>
        {:else}
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <polygon points="7 4 20 12 7 20 7 4" />
          </svg>
        {/if}
      </button>
    </StopwatchDial>
  </div>
</section>

<style>
  .focus-widget {
    width: 100vw;
    height: 100vh;
    box-sizing: border-box;
    overflow: hidden;
    background-color: var(--bg-app);
    border: 1px solid var(--border-card);
    padding: 10px 12px 12px;
    display: flex;
    flex-direction: column;
    gap: 4px;
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
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.25;
  }

  .btn-exit {
    background-color: transparent;
    color: var(--text-dim);
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
    background-color: var(--bg-hover);
    color: var(--text-primary);
    border-color: var(--border-subtle);
  }

  .focus-body {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 0;
  }

  .focus-time {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-variant-numeric: tabular-nums;
    font-feature-settings: "tnum" 1;
    font-size: 1.5rem;
    font-weight: 700;
    line-height: 1.1;
    letter-spacing: -0.02em;
    color: var(--text-muted);
  }

  .focus-time[data-status="running"] {
    color: var(--text-strong);
  }

  .focus-time[data-status="paused"] {
    color: var(--text-secondary);
  }

  .btn-action {
    width: 32px;
    height: 32px;
    padding: 0;
    background-color: var(--accent-bg);
    color: var(--accent-fg);
    border: 1px solid var(--accent-bg);
    border-radius: 50%;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    flex-shrink: 0;
    transition:
      background-color 120ms ease,
      color 120ms ease,
      border-color 120ms ease;
  }

  .btn-action:hover {
    background-color: var(--accent-bg-hover);
    border-color: var(--accent-bg-hover);
  }

  .btn-action[data-status="running"] {
    background-color: var(--bg-control);
    color: var(--text-primary);
    border: 1px solid var(--border-strong);
  }

  .btn-action[data-status="running"]:hover {
    background-color: var(--bg-control-hover);
    border-color: var(--border-strong);
  }
</style>
