<script lang="ts">
  import { tick } from "svelte";
  import type { StopwatchDto } from "../api";
  import { computeElapsedMs, formatElapsed } from "../time";
  import { settingsStore } from "../stores/settings.svelte";
  import StopwatchDial from "./StopwatchDial.svelte";

  interface Props {
    stopwatch: StopwatchDto;
    nowMs: number;
    editMode?: boolean;
    onStart: (id: number) => Promise<void> | void;
    onPause: (id: number) => Promise<void> | void;
    onReset: (id: number) => Promise<void> | void;
    onRename: (id: number, name: string) => Promise<boolean> | boolean;
    onDelete: (id: number) => Promise<void> | void;
    onFocus?: (id: number) => void;
  }

  let {
    stopwatch,
    nowMs,
    editMode = false,
    onStart,
    onPause,
    onReset,
    onRename,
    onDelete,
    onFocus,
  }: Props = $props();

  let isEditing = $state(false);
  let draftName = $state("");
  let confirmMode = $state<"none" | "reset" | "delete">("none");
  let nameInputEl = $state<HTMLInputElement | null>(null);

  let elapsedMs = $derived(computeElapsedMs(stopwatch, nowMs));
  let formattedTime = $derived(formatElapsed(elapsedMs, settingsStore.showStopwatchTenths));

  let primaryActionLabel = $derived(
    stopwatch.status === "running"
      ? "Pause"
      : stopwatch.status === "paused"
        ? "Resume"
        : "Start"
  );

  let isResetDisabled = $derived(elapsedMs === 0 && stopwatch.status === "idle");

  $effect(() => {
    if (!editMode) {
      isEditing = false;
      if (confirmMode === "delete") {
        confirmMode = "none";
      }
    }
  });

  async function beginEditing(): Promise<void> {
    confirmMode = "none";
    draftName = stopwatch.name;
    isEditing = true;
    await tick();
    if (nameInputEl) {
      nameInputEl.focus();
      nameInputEl.select();
    }
  }

  function cancelEditing(): void {
    isEditing = false;
    draftName = stopwatch.name;
  }

  async function submitRename(): Promise<void> {
    if (!isEditing) return;
    const trimmed = draftName.trim();
    if (trimmed === stopwatch.name) {
      isEditing = false;
      return;
    }
    const ok = await onRename(stopwatch.id, draftName);
    if (ok) {
      isEditing = false;
    }
  }

  async function handleEditKeydown(event: KeyboardEvent): Promise<void> {
    if (event.key === "Enter") {
      event.preventDefault();
      await submitRename();
    } else if (event.key === "Escape") {
      event.preventDefault();
      cancelEditing();
    }
  }

  async function handlePrimaryAction(): Promise<void> {
    confirmMode = "none";
    if (stopwatch.status === "running") {
      await onPause(stopwatch.id);
    } else {
      await onStart(stopwatch.id);
    }
  }

  async function handleResetClick(): Promise<void> {
    if (isResetDisabled) return;
    if (elapsedMs > 0 && settingsStore.confirmStopwatchReset) {
      confirmMode = "reset";
    } else {
      await onReset(stopwatch.id);
    }
  }

  async function confirmReset(): Promise<void> {
    confirmMode = "none";
    await onReset(stopwatch.id);
  }

  function requestDelete(): void {
    confirmMode = "delete";
  }

  async function confirmDelete(): Promise<void> {
    confirmMode = "none";
    await onDelete(stopwatch.id);
  }

  function cancelConfirm(): void {
    confirmMode = "none";
  }

  function handleFocusClick(): void {
    confirmMode = "none";
    onFocus?.(stopwatch.id);
  }
</script>

<article class="stopwatch-card">
  <header class="card-header">
    {#if isEditing}
      <div class="rename-row">
        <input
          bind:this={nameInputEl}
          type="text"
          class="rename-input"
          maxlength="50"
          bind:value={draftName}
          onkeydown={handleEditKeydown}
          aria-label="Rename stopwatch"
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
          onclick={cancelEditing}
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
            class="stopwatch-name"
            ondblclick={() => {
              if (editMode) void beginEditing();
            }}
            title={editMode ? "Double-click to rename" : stopwatch.name}
          >
            {stopwatch.name}
          </h2>
          {#if editMode}
            <button
              type="button"
              class="btn-icon-ghost"
              onclick={beginEditing}
              aria-label="Rename stopwatch {stopwatch.name}"
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
            aria-label="Delete stopwatch {stopwatch.name}"
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

  <StopwatchDial status={stopwatch.status} {elapsedMs} size={208}>
    <div class="time-display" data-status={stopwatch.status}>
      <span class="time-main"
        >{formattedTime.hours}:{formattedTime.minutes}:{formattedTime.seconds}</span
      >{#if settingsStore.showStopwatchTenths}<span class="time-tenths">.{formattedTime.tenths}</span>{/if}
    </div>
  </StopwatchDial>

  {#if confirmMode === "reset"}
    <div class="confirm-bar" role="region" aria-label="Confirm reset stopwatch">
      <span class="confirm-text">Reset time to 00:00:00?</span>
      <div class="confirm-actions">
        <button type="button" class="btn-confirm-danger" onclick={confirmReset}>
          Yes, Reset
        </button>
        <button type="button" class="btn-confirm-cancel" onclick={cancelConfirm}>
          Cancel
        </button>
      </div>
    </div>
  {:else if confirmMode === "delete"}
    <div class="confirm-bar" role="region" aria-label="Confirm delete stopwatch">
      <span class="confirm-text">Delete this stopwatch?</span>
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
        data-status={stopwatch.status}
        onclick={handlePrimaryAction}
        aria-label="{primaryActionLabel} stopwatch {stopwatch.name}"
        title={primaryActionLabel}
      >
        {#if stopwatch.status === "running"}
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
        aria-label="Reset stopwatch {stopwatch.name}"
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
        onclick={handleFocusClick}
        aria-label="Focus mode for {stopwatch.name}"
        title="Focus mode"
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
          <rect x="3" y="3" width="18" height="18" rx="2" />
          <rect x="12" y="4" width="8" height="6" rx="1" fill="currentColor" stroke="none" />
        </svg>
      </button>
    </div>
  {/if}
</article>

<style>
  .stopwatch-card {
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

  .stopwatch-name {
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

  .time-tenths {
    font-size: 1.05rem;
    font-weight: 500;
    color: var(--text-dim);
  }

  .time-display[data-status="running"] .time-tenths {
    color: var(--text-muted);
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
