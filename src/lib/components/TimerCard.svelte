<script lang="ts">
  import { tick } from "svelte";
  import type { TimerDto } from "../api";
  import {
    computeRemainingMs,
    formatElapsed,
    partsToMs,
    splitDurationMs,
  } from "../time";
  import StopwatchDial from "./StopwatchDial.svelte";

  interface Props {
    timer: TimerDto;
    nowMs: number;
    editMode?: boolean;
    onStart: (id: number) => Promise<void> | void;
    onPause: (id: number) => Promise<void> | void;
    onReset: (id: number) => Promise<void> | void;
    onRename: (id: number, name: string) => Promise<boolean> | boolean;
    onSetDuration: (id: number, durationMs: number) => Promise<boolean> | boolean;
    onDelete: (id: number) => Promise<void> | void;
  }

  let {
    timer,
    nowMs,
    editMode = false,
    onStart,
    onPause,
    onReset,
    onRename,
    onSetDuration,
    onDelete,
  }: Props = $props();

  let isEditingName = $state(false);
  let draftName = $state("");
  let isEditingDuration = $state(false);
  let draftHours = $state(0);
  let draftMinutes = $state(5);
  let draftSeconds = $state(0);
  let confirmMode = $state<"none" | "delete">("none");
  let nameInputEl = $state<HTMLInputElement | null>(null);

  const PRESETS = [
    { label: "1m", ms: 60_000 },
    { label: "5m", ms: 5 * 60_000 },
    { label: "10m", ms: 10 * 60_000 },
    { label: "15m", ms: 15 * 60_000 },
    { label: "25m", ms: 25 * 60_000 },
    { label: "1h", ms: 60 * 60_000 },
  ];

  let remainingMs = $derived(computeRemainingMs(timer, nowMs));
  // Display rounded up to whole seconds while running so 00:05:00 ticks cleanly to 00:04:59
  let displayMs = $derived(
    timer.status === "running" && remainingMs > 0
      ? Math.ceil(remainingMs / 1000) * 1000
      : remainingMs
  );
  let formattedTime = $derived(formatElapsed(displayMs, false));
  let targetFormatted = $derived(formatElapsed(timer.duration_ms, false).formatted);
  let progress = $derived(
    timer.duration_ms > 0 ? Math.max(0, Math.min(1, remainingMs / timer.duration_ms)) : 0
  );

  let primaryActionLabel = $derived(
    timer.status === "running"
      ? "Pause"
      : timer.status === "paused"
        ? "Resume"
        : timer.status === "completed"
          ? "Restart"
          : "Start"
  );

  let isResetDisabled = $derived(
    timer.status === "idle" && timer.remaining_ms === timer.duration_ms
  );

  $effect(() => {
    if (!editMode) {
      isEditingName = false;
      if (confirmMode === "delete") {
        confirmMode = "none";
      }
    }
  });

  async function beginEditingName(): Promise<void> {
    confirmMode = "none";
    isEditingDuration = false;
    draftName = timer.name;
    isEditingName = true;
    await tick();
    if (nameInputEl) {
      nameInputEl.focus();
      nameInputEl.select();
    }
  }

  function cancelEditingName(): void {
    isEditingName = false;
    draftName = timer.name;
  }

  async function submitRename(): Promise<void> {
    if (!isEditingName) return;
    const trimmed = draftName.trim();
    if (trimmed === timer.name) {
      isEditingName = false;
      return;
    }
    const ok = await onRename(timer.id, draftName);
    if (ok) {
      isEditingName = false;
    }
  }

  async function handleEditKeydown(event: KeyboardEvent): Promise<void> {
    if (event.key === "Enter") {
      event.preventDefault();
      await submitRename();
    } else if (event.key === "Escape") {
      event.preventDefault();
      cancelEditingName();
    }
  }

  function openDurationEditor(): void {
    if (timer.status === "running") return;
    confirmMode = "none";
    isEditingName = false;
    const parts = splitDurationMs(timer.duration_ms);
    draftHours = parts.hours;
    draftMinutes = parts.minutes;
    draftSeconds = parts.seconds;
    isEditingDuration = true;
  }

  function cancelDurationEditor(): void {
    isEditingDuration = false;
  }

  async function submitDuration(): Promise<void> {
    const ms = partsToMs(draftHours, draftMinutes, draftSeconds);
    if (ms < 1000) return;
    const ok = await onSetDuration(timer.id, ms);
    if (ok) {
      isEditingDuration = false;
    }
  }

  async function applyPreset(ms: number): Promise<void> {
    const ok = await onSetDuration(timer.id, ms);
    if (ok) {
      isEditingDuration = false;
    }
  }

  async function handlePrimaryAction(): Promise<void> {
    confirmMode = "none";
    isEditingDuration = false;
    if (timer.status === "running") {
      await onPause(timer.id);
    } else {
      await onStart(timer.id);
    }
  }

  async function handleResetClick(): Promise<void> {
    if (isResetDisabled) return;
    confirmMode = "none";
    await onReset(timer.id);
  }

  function requestDelete(): void {
    confirmMode = "delete";
  }

  async function confirmDelete(): Promise<void> {
    confirmMode = "none";
    await onDelete(timer.id);
  }

  function cancelConfirm(): void {
    confirmMode = "none";
  }
</script>

<article class="timer-card">
  <header class="card-header">
    {#if isEditingName}
      <div class="rename-row">
        <input
          bind:this={nameInputEl}
          type="text"
          class="rename-input"
          maxlength="50"
          bind:value={draftName}
          onkeydown={handleEditKeydown}
          aria-label="Rename timer"
        />
        <button
          type="button"
          class="btn-inline btn-inline-save"
          onclick={submitRename}
          aria-label="Save name"
          title="Save (Enter)"
        >
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <polyline points="20 6 9 17 4 12" />
          </svg>
        </button>
        <button
          type="button"
          class="btn-inline"
          onclick={cancelEditingName}
          aria-label="Cancel rename"
          title="Cancel (Esc)"
        >
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>
    {:else}
      <div class="header-row">
        <div class="name-row">
          <h2
            class="timer-name"
            ondblclick={() => {
              if (editMode) void beginEditingName();
            }}
            title={editMode ? "Double-click to rename" : timer.name}
          >
            {timer.name}
          </h2>
          {#if editMode}
            <button
              type="button"
              class="btn-icon-ghost"
              onclick={beginEditingName}
              aria-label="Rename timer {timer.name}"
              title="Rename"
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
                <path d="M12 20h9" />
                <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4Z" />
              </svg>
            </button>
          {/if}
        </div>

        {#if editMode}
          <button
            type="button"
            class="btn-icon-ghost btn-delete-trigger"
            onclick={requestDelete}
            aria-label="Delete timer {timer.name}"
            title="Delete"
          >
            <svg
              width="15"
              height="15"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              aria-hidden="true"
            >
              <polyline points="3 6 5 6 21 6" />
              <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
              <path d="M10 11v6" />
              <path d="M14 11v6" />
              <path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2" />
            </svg>
          </button>
        {/if}
      </div>
    {/if}
  </header>

  {#if isEditingDuration}
    <div class="duration-editor" role="region" aria-label="Set timer duration">
      <div class="duration-inputs">
        <label class="duration-field">
          <span class="duration-unit">H</span>
          <input
            type="number"
            min="0"
            max="99"
            bind:value={draftHours}
            class="duration-num"
          />
        </label>
        <span class="duration-sep">:</span>
        <label class="duration-field">
          <span class="duration-unit">M</span>
          <input
            type="number"
            min="0"
            max="59"
            bind:value={draftMinutes}
            class="duration-num"
          />
        </label>
        <span class="duration-sep">:</span>
        <label class="duration-field">
          <span class="duration-unit">S</span>
          <input
            type="number"
            min="0"
            max="59"
            bind:value={draftSeconds}
            class="duration-num"
          />
        </label>
      </div>

      <div class="preset-row">
        {#each PRESETS as preset (preset.label)}
          <button
            type="button"
            class="preset-btn"
            onclick={() => applyPreset(preset.ms)}
          >
            {preset.label}
          </button>
        {/each}
      </div>

      <div class="duration-actions">
        <button
          type="button"
          class="btn-confirm-cancel"
          onclick={cancelDurationEditor}
        >
          Cancel
        </button>
        <button
          type="button"
          class="btn-duration-save"
          disabled={partsToMs(draftHours, draftMinutes, draftSeconds) < 1000}
          onclick={submitDuration}
        >
          Set
        </button>
      </div>
    </div>
  {:else}
    <StopwatchDial status={timer.status} {progress} size={208}>
      <div class="time-display" data-status={timer.status}>
        {formattedTime.formatted}
      </div>
      {#if timer.status === "completed"}
        <span class="status-note status-note-done">Time's up</span>
      {:else}
        <span class="status-note">Target {targetFormatted}</span>
      {/if}
    </StopwatchDial>
  {/if}

  {#if confirmMode === "delete"}
    <div class="confirm-bar" role="region" aria-label="Confirm delete timer">
      <span class="confirm-text">Delete this timer?</span>
      <div class="confirm-actions">
        <button type="button" class="btn-confirm-danger" onclick={confirmDelete}>
          Yes, Delete
        </button>
        <button type="button" class="btn-confirm-cancel" onclick={cancelConfirm}>
          Cancel
        </button>
      </div>
    </div>
  {:else}
    <div class="controls-row">
      <button
        type="button"
        class="btn-circle-primary"
        data-status={timer.status}
        onclick={handlePrimaryAction}
        aria-label="{primaryActionLabel} timer {timer.name}"
        title={primaryActionLabel}
      >
        {#if timer.status === "running"}
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <rect x="6" y="4" width="4" height="16" rx="1" />
            <rect x="14" y="4" width="4" height="16" rx="1" />
          </svg>
        {:else}
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <polygon points="7 4 20 12 7 20 7 4" />
          </svg>
        {/if}
      </button>

      <button
        type="button"
        class="btn-circle-secondary"
        disabled={isResetDisabled}
        onclick={handleResetClick}
        aria-label="Reset timer {timer.name}"
        title="Reset"
      >
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="M3 12a9 9 0 1 0 3-6.7L3 8" />
          <path d="M3 3v5h5" />
        </svg>
      </button>

      <button
        type="button"
        class="btn-circle-secondary"
        disabled={timer.status === "running"}
        onclick={() => (isEditingDuration ? cancelDurationEditor() : openDurationEditor())}
        aria-label="Set duration for {timer.name}"
        title="Set duration"
      >
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <circle cx="12" cy="12" r="9" />
          <polyline points="12 7 12 12 15.5 14" />
        </svg>
      </button>
    </div>
  {/if}
</article>

<style>
  .timer-card {
    background-color: var(--bg-card);
    border: 1px solid var(--border-card);
    border-radius: 10px;
    padding: 14px 18px 20px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 18px;
    min-width: 0;
  }

  .card-header {
    display: flex;
    align-items: center;
    min-width: 0;
    min-height: 32px;
  }

  .header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    width: 100%;
    min-width: 0;
  }

  .name-row {
    display: flex;
    align-items: center;
    gap: 4px;
    min-width: 0;
    flex: 1;
  }

  .timer-name {
    margin: 0;
    font-size: 1.0625rem;
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    cursor: pointer;
    user-select: none;
    min-width: 0;
  }

  .rename-row {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    min-width: 0;
    min-height: 32px;
  }

  .rename-input {
    flex: 1;
    min-width: 0;
    width: 100%;
    background-color: var(--bg-app);
    color: var(--text-primary);
    border: 1px solid var(--border-strong);
    border-radius: 6px;
    padding: 6px 10px;
    font-size: 0.9375rem;
    font-family: inherit;
    outline: none;
    transition: border-color 120ms ease;
  }

  .rename-input:focus-visible {
    border-color: var(--text-muted);
  }

  .time-display {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-variant-numeric: tabular-nums;
    font-feature-settings: "tnum" 1;
    font-size: 1.625rem;
    font-weight: 700;
    line-height: 1.15;
    letter-spacing: -0.02em;
    color: var(--text-muted);
    user-select: none;
    white-space: nowrap;
    text-align: center;
  }

  .time-display[data-status="running"] {
    color: var(--text-strong);
  }

  .time-display[data-status="paused"] {
    color: var(--text-secondary);
  }

  .time-display[data-status="completed"] {
    color: var(--text-primary);
  }

  .status-note {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 0.75rem;
    color: var(--text-dim);
    user-select: none;
  }

  .status-note-done {
    color: var(--danger-fg);
    font-family: inherit;
    font-weight: 600;
  }

  .duration-editor {
    height: 208px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 14px;
    background-color: var(--bg-surface);
    border: 1px solid var(--border-card);
    border-radius: 8px;
    padding: 14px;
  }

  .duration-inputs {
    display: flex;
    align-items: flex-end;
    gap: 6px;
  }

  .duration-field {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }

  .duration-unit {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 0.6875rem;
    color: var(--text-dim);
    letter-spacing: 0.06em;
  }

  .duration-num {
    width: 52px;
    background-color: var(--bg-app);
    color: var(--text-primary);
    border: 1px solid var(--border-control);
    border-radius: 6px;
    padding: 6px 4px;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 1.125rem;
    font-weight: 600;
    text-align: center;
    outline: none;
  }

  .duration-num:focus-visible {
    border-color: var(--text-muted);
  }

  .duration-sep {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--text-dim);
    padding-bottom: 7px;
  }

  .preset-row {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 6px;
  }

  .preset-btn {
    background-color: var(--bg-card);
    color: var(--text-secondary);
    border: 1px solid var(--border-card);
    border-radius: 6px;
    padding: 3px 8px;
    font-size: 0.75rem;
    font-weight: 500;
  }

  .preset-btn:hover {
    background-color: var(--bg-control-hover);
    color: var(--text-strong);
    border-color: var(--border-strong);
  }

  .duration-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .btn-duration-save {
    background-color: var(--accent-bg);
    color: var(--accent-fg);
    border: 1px solid var(--accent-bg);
    border-radius: 6px;
    padding: 5px 14px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .btn-duration-save:hover:not(:disabled) {
    background-color: var(--accent-bg-hover);
    border-color: var(--accent-bg-hover);
  }

  .controls-row {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
  }

  button {
    font-family: inherit;
    cursor: pointer;
    transition:
      background-color 120ms ease,
      color 120ms ease,
      border-color 120ms ease;
  }

  button:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .btn-circle-primary {
    width: 42px;
    height: 42px;
    border-radius: 50%;
    background-color: var(--accent-bg);
    color: var(--accent-fg);
    border: 1px solid var(--accent-bg);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    flex-shrink: 0;
  }

  .btn-circle-primary:hover:not(:disabled) {
    background-color: var(--accent-bg-hover);
    border-color: var(--accent-bg-hover);
  }

  .btn-circle-primary[data-status="running"] {
    background-color: var(--bg-control);
    color: var(--text-primary);
    border: 1px solid var(--border-strong);
  }

  .btn-circle-primary[data-status="running"]:hover:not(:disabled) {
    background-color: var(--bg-control-hover);
    border-color: var(--border-strong);
  }

  .btn-circle-secondary {
    width: 38px;
    height: 38px;
    border-radius: 50%;
    background-color: var(--bg-control);
    color: var(--text-secondary);
    border: 1px solid var(--border-control);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    flex-shrink: 0;
  }

  .btn-circle-secondary:hover:not(:disabled) {
    background-color: var(--bg-control-hover);
    color: var(--text-strong);
    border-color: var(--border-strong);
  }

  .btn-icon-ghost {
    width: 26px;
    height: 26px;
    border-radius: 6px;
    background-color: transparent;
    color: var(--text-muted);
    border: none;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    flex-shrink: 0;
  }

  .btn-icon-ghost:hover {
    background-color: transparent;
    color: var(--text-primary);
  }

  .btn-delete-trigger:hover {
    color: var(--danger-fg);
  }

  .btn-inline {
    width: 30px;
    height: 30px;
    border-radius: 6px;
    background-color: var(--bg-control);
    color: var(--text-secondary);
    border: 1px solid var(--border-control);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    flex-shrink: 0;
  }

  .btn-inline:hover {
    background-color: var(--bg-control-hover);
    color: var(--text-strong);
    border-color: var(--border-strong);
  }

  .btn-inline-save {
    background-color: var(--accent-bg);
    color: var(--accent-fg);
    border-color: var(--accent-bg);
  }

  .btn-inline-save:hover {
    background-color: var(--accent-bg-hover);
    color: var(--accent-fg);
    border-color: var(--accent-bg-hover);
  }

  .confirm-bar {
    display: flex;
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
    background-color: var(--bg-app);
    border: 1px solid var(--border-card);
    border-radius: 8px;
    padding: 10px 12px;
  }

  .confirm-text {
    font-size: 0.8125rem;
    color: var(--text-primary);
  }

  .confirm-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
  }

  .btn-confirm-danger {
    background-color: var(--bg-control);
    color: var(--danger-fg);
    border: 1px solid var(--border-control);
    border-radius: 6px;
    padding: 5px 12px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .btn-confirm-danger:hover {
    background-color: var(--bg-control-hover);
    border-color: var(--border-strong);
  }

  .btn-confirm-cancel {
    background-color: var(--bg-control);
    color: var(--text-secondary);
    border: 1px solid var(--border-control);
    border-radius: 6px;
    padding: 5px 12px;
    font-size: 0.75rem;
    font-weight: 500;
  }

  .btn-confirm-cancel:hover {
    background-color: var(--bg-control-hover);
    color: var(--text-strong);
    border-color: var(--border-strong);
  }
</style>
