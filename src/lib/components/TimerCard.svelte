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

