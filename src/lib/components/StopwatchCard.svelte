<script lang="ts">
  import { tick } from "svelte";
  import type { StopwatchDto } from "../api";
  import { computeElapsedMs, formatElapsed } from "../time";

  interface Props {
    stopwatch: StopwatchDto;
    nowMs: number;
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
  let formattedTime = $derived(formatElapsed(elapsedMs, true));

  let statusLabel = $derived(
    stopwatch.status === "running"
      ? "BERJALAN"
      : stopwatch.status === "paused"
        ? "DIJEDA"
        : "SIAP"
  );

  let primaryActionLabel = $derived(
    stopwatch.status === "running"
      ? "Pause"
      : stopwatch.status === "paused"
        ? "Resume"
        : "Start"
  );

  let isResetDisabled = $derived(elapsedMs === 0 && stopwatch.status === "idle");

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
    if (elapsedMs > 0) {
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
    <div class="title-group">
      <span class="status-label" data-status={stopwatch.status}>
        {statusLabel}
      </span>

      {#if isEditing}
        <div class="rename-row">
          <input
            bind:this={nameInputEl}
            type="text"
            class="rename-input"
            maxlength="50"
            bind:value={draftName}
            onkeydown={handleEditKeydown}
            aria-label="Ubah nama stopwatch"
          />
          <button
            type="button"
            class="btn-inline"
            onclick={submitRename}
            title="Simpan nama (Enter)"
          >
            Simpan
          </button>
          <button
            type="button"
            class="btn-inline btn-muted"
            onclick={cancelEditing}
            title="Batal (Esc)"
          >
            Batal
          </button>
        </div>
      {:else}
        <div class="name-row">
          <h2
            class="stopwatch-name"
            ondblclick={beginEditing}
            title="Klik dua kali untuk mengubah nama"
          >
            {stopwatch.name}
          </h2>
          <button
            type="button"
            class="btn-ghost"
            onclick={beginEditing}
            aria-label="Ubah nama stopwatch {stopwatch.name}"
          >
            Ubah Nama
          </button>
        </div>
      {/if}
    </div>

    <button
      type="button"
      class="btn-ghost btn-delete-trigger"
      onclick={requestDelete}
      aria-label="Hapus stopwatch {stopwatch.name}"
    >
      Hapus
    </button>
  </header>

  <div class="time-display" data-status={stopwatch.status}>
    <span class="time-main"
      >{formattedTime.hours}:{formattedTime.minutes}:{formattedTime.seconds}</span
    ><span class="time-tenths">.{formattedTime.tenths}</span>
  </div>

  {#if confirmMode === "reset"}
    <div class="confirm-bar" role="region" aria-label="Konfirmasi reset stopwatch">
      <span class="confirm-text">Reset waktu ke 00:00:00?</span>
      <div class="confirm-actions">
        <button type="button" class="btn-confirm-danger" onclick={confirmReset}>
          Ya, Reset
        </button>
        <button type="button" class="btn-secondary" onclick={cancelConfirm}>
          Batal
        </button>
      </div>
    </div>
  {:else if confirmMode === "delete"}
    <div class="confirm-bar" role="region" aria-label="Konfirmasi hapus stopwatch">
      <span class="confirm-text">Hapus stopwatch ini?</span>
      <div class="confirm-actions">
        <button type="button" class="btn-confirm-danger" onclick={confirmDelete}>
          Ya, Hapus
        </button>
        <button type="button" class="btn-secondary" onclick={cancelConfirm}>
          Batal
        </button>
      </div>
    </div>
  {:else}
    <div class="controls-row">
      <button
        type="button"
        class="btn-primary"
        data-status={stopwatch.status}
        onclick={handlePrimaryAction}
      >
        {primaryActionLabel}
      </button>

      <button
        type="button"
        class="btn-secondary"
        disabled={isResetDisabled}
        onclick={handleResetClick}
      >
        Reset
      </button>

      <button
        type="button"
        class="btn-secondary"
        onclick={handleFocusClick}
        aria-label="Mode fokus untuk {stopwatch.name}"
      >
        Focus
      </button>
    </div>
  {/if}
</article>

<style>
  .stopwatch-card {
    background-color: #111111;
    border: 1px solid #262626;
    border-radius: 8px;
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .card-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 8px;
  }

  .title-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
    flex: 1;
  }

  .status-label {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 0.6875rem;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: #737373;
    line-height: 1.2;
  }

  .status-label[data-status="running"] {
    color: #10b981;
  }

  .status-label[data-status="paused"] {
    color: #d97706;
  }

  .name-row {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .stopwatch-name {
    margin: 0;
    font-size: 0.9375rem;
    font-weight: 600;
    color: #f5f5f5;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    cursor: pointer;
    user-select: none;
  }

  .rename-row {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
  }

  .rename-input {
    flex: 1;
    min-width: 0;
    background-color: #171717;
    color: #f5f5f5;
    border: 1px solid #262626;
    border-radius: 6px;
    padding: 4px 8px;
    font-size: 0.875rem;
    font-family: inherit;
    outline: none;
    transition: border-color 120ms ease;
  }

  .rename-input:focus-visible {
    border-color: #525252;
  }

  .time-display {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-variant-numeric: tabular-nums;
    font-feature-settings: "tnum" 1;
    font-size: 2rem;
    font-weight: 700;
    line-height: 1.15;
    letter-spacing: -0.02em;
    color: #a3a3a3;
    user-select: none;
  }

  .time-display[data-status="running"] {
    color: #ffffff;
  }

  .time-display[data-status="paused"] {
    color: #e5e5e5;
  }

  .time-tenths {
    font-size: 1.25rem;
    font-weight: 500;
    color: #737373;
  }

  .time-display[data-status="running"] .time-tenths {
    color: #a3a3a3;
  }

  .controls-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  button {
    font-family: inherit;
    font-size: 0.8125rem;
    font-weight: 500;
    border-radius: 6px;
    cursor: pointer;
    transition:
      background-color 120ms ease,
      color 120ms ease,
      border-color 120ms ease;
  }

  button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .btn-primary {
    flex: 1;
    background-color: #f5f5f5;
    color: #0a0a0a;
    border: 1px solid #f5f5f5;
    padding: 7px 12px;
    font-weight: 600;
  }

  .btn-primary:hover:not(:disabled) {
    background-color: #e5e5e5;
    border-color: #e5e5e5;
  }

  .btn-primary[data-status="running"] {
    background-color: #171717;
    color: #f5f5f5;
    border: 1px solid #404040;
  }

  .btn-primary[data-status="running"]:hover:not(:disabled) {
    background-color: #1f1f1f;
    border-color: #525252;
  }

  .btn-secondary {
    background-color: #171717;
    color: #a3a3a3;
    border: 1px solid #262626;
    padding: 7px 12px;
  }

  .btn-secondary:hover:not(:disabled) {
    background-color: #1f1f1f;
    color: #f5f5f5;
    border-color: #404040;
  }

  .btn-ghost {
    background-color: transparent;
    color: #737373;
    border: 1px solid transparent;
    padding: 2px 6px;
    font-size: 0.75rem;
    flex-shrink: 0;
  }

  .btn-ghost:hover {
    background-color: #171717;
    color: #a3a3a3;
    border-color: #262626;
  }

  .btn-delete-trigger:hover {
    color: #f87171;
  }

  .btn-inline {
    background-color: #171717;
    color: #f5f5f5;
    border: 1px solid #262626;
    padding: 4px 8px;
    font-size: 0.75rem;
    flex-shrink: 0;
  }

  .btn-inline:hover {
    background-color: #1f1f1f;
    border-color: #404040;
  }

  .btn-muted {
    color: #a3a3a3;
  }

  .confirm-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    background-color: #171717;
    border: 1px solid #262626;
    border-radius: 6px;
    padding: 6px 10px;
  }

  .confirm-text {
    font-size: 0.8125rem;
    color: #e5e5e5;
  }

  .confirm-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .btn-confirm-danger {
    background-color: #171717;
    color: #f87171;
    border: 1px solid #262626;
    padding: 5px 10px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .btn-confirm-danger:hover {
    background-color: #1f1f1f;
    border-color: #404040;
  }
</style>
