<script lang="ts">
  import { tick } from "svelte";
  import type { AlarmDto } from "../api";
  import {
    REPEAT_DAY_OPTIONS,
    formatAlarmCountdown,
    formatAlarmTime,
    formatRepeatDays,
  } from "../alarm";

  interface Props {
    alarm: AlarmDto;
    nowMs: number;
    editMode?: boolean;
    onToggle: (id: number, enabled: boolean) => Promise<void> | void;
    onUpdate: (
      id: number,
      name: string,
      hour: number,
      minute: number,
      repeatDays: number
    ) => Promise<boolean> | boolean;
    onDelete: (id: number) => Promise<void> | void;
  }

  let { alarm, nowMs, editMode = false, onToggle, onUpdate, onDelete }: Props = $props();

  let isEditing = $state(false);
  let draftName = $state("");
  let draftTime = $state("07:00");
  let confirmMode = $state<"none" | "delete">("none");
  let isDayUpdating = $state(false);
  let nameInputEl = $state<HTMLInputElement | null>(null);

  let parsedTime = $derived.by(() => {
    const match = /^([01]\d|2[0-3]):([0-5]\d)$/.exec(draftTime);
    if (!match) return null;
    return { hour: Number(match[1]), minute: Number(match[2]) };
  });

  let trimmedName = $derived(draftName.trim());
  let canSave = $derived(trimmedName.length > 0 && parsedTime !== null);

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
    draftName = alarm.name;
    draftTime = formatAlarmTime(alarm.hour, alarm.minute);
    isEditing = true;
    await tick();
    if (nameInputEl) {
      nameInputEl.focus();
      nameInputEl.select();
    }
  }

  function cancelEditing(): void {
    isEditing = false;
    confirmMode = "none";
  }

  async function submitEdit(): Promise<void> {
    if (!isEditing || !canSave || parsedTime === null) return;
    const ok = await onUpdate(
      alarm.id,
      trimmedName,
      parsedTime.hour,
      parsedTime.minute,
      alarm.repeat_days
    );
    if (ok) {
      isEditing = false;
    }
  }

  async function toggleDay(bit: number): Promise<void> {
    if (!editMode || isDayUpdating) return;
    const nextMask =
      (alarm.repeat_days & bit) !== 0 ? alarm.repeat_days & ~bit : alarm.repeat_days | bit;
    isDayUpdating = true;
    try {
      await onUpdate(alarm.id, alarm.name, alarm.hour, alarm.minute, nextMask);
    } finally {
      isDayUpdating = false;
    }
  }

  function handleEditorKeydown(event: KeyboardEvent): void {
    if (isEditing && event.key === "Escape") {
      event.preventDefault();
      cancelEditing();
    }
  }

  async function handleToggle(): Promise<void> {
    await onToggle(alarm.id, !alarm.enabled);
  }

  function requestDelete(): void {
    isEditing = false;
    confirmMode = "delete";
  }

  async function confirmDelete(): Promise<void> {
    confirmMode = "none";
    await onDelete(alarm.id);
  }

  function cancelConfirm(): void {
    confirmMode = "none";
  }
</script>

<svelte:window onkeydown={handleEditorKeydown} />

<article class="alarm-card" class:alarm-card-off={!alarm.enabled}>
  <div class="alarm-card-top">
    <div class="alarm-time-block">
      <span class="alarm-time">{formatAlarmTime(alarm.hour, alarm.minute)}</span>
      <div class="alarm-status-line">
        <svg
          class="alarm-bell"
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
          <path d="M18 8a6 6 0 0 0-12 0c0 7-3 9-3 9h18s-3-2-3-9" />
          <path d="M13.73 21a2 2 0 0 1-3.46 0" />
        </svg>
        <span class="alarm-next">{formatAlarmCountdown(alarm, nowMs)}</span>
      </div>
    </div>

    <div class="alarm-actions">
      {#if editMode}
        <button
          type="button"
          class="btn-icon-ghost"
          onclick={beginEditing}
          aria-label="Edit alarm {alarm.name}"
          title="Edit"
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
            <path d="M12 20h9" />
            <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4Z" />
          </svg>
        </button>
        <button
          type="button"
          class="btn-icon-ghost btn-delete-trigger"
          onclick={requestDelete}
          aria-label="Delete alarm {alarm.name}"
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

      <button
        type="button"
        role="switch"
        aria-checked={alarm.enabled}
        aria-label="Enable alarm {alarm.name}"
        class="toggle-switch"
        class:toggle-switch-on={alarm.enabled}
        onclick={handleToggle}
      >
        <span class="toggle-thumb"></span>
      </button>
    </div>
  </div>

  <h2 class="alarm-name" title={alarm.name}>{alarm.name}</h2>

  <div
    class="alarm-days"
    role="group"
    aria-label={editMode
      ? "Repeat days"
      : `Repeats: ${formatRepeatDays(alarm.repeat_days)}`}
  >
    {#each REPEAT_DAY_OPTIONS as day (day.bit)}
      {#if editMode}
        <button
          type="button"
          class="alarm-day"
          class:alarm-day-on={(alarm.repeat_days & day.bit) !== 0}
          aria-pressed={(alarm.repeat_days & day.bit) !== 0}
          aria-label={day.label}
          title={day.label}
          disabled={isDayUpdating}
          onclick={() => toggleDay(day.bit)}
        >
          {day.short}
        </button>
      {:else}
        <span
          class="alarm-day"
          class:alarm-day-on={(alarm.repeat_days & day.bit) !== 0}
          aria-hidden="true"
          title={day.label}
        >
          {day.short}
        </span>
      {/if}
    {/each}
  </div>

  {#if isEditing}
    <div class="alarm-editor" role="region" aria-label="Edit alarm {alarm.name}">
      <div class="alarm-editor-fields">
        <label class="alarm-field">
          <span class="alarm-field-label">Name</span>
          <input
            bind:this={nameInputEl}
            type="text"
            class="alarm-text-input"
            maxlength="50"
            bind:value={draftName}
            onkeydown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                void submitEdit();
              }
            }}
          />
        </label>
        <label class="alarm-field alarm-field-time">
          <span class="alarm-field-label">Time</span>
          <input
            type="time"
            class="alarm-time-input"
            bind:value={draftTime}
            onkeydown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                void submitEdit();
              }
            }}
          />
        </label>
      </div>

      <p class="alarm-editor-error" class:alarm-editor-error-visible={!canSave}>
        {trimmedName.length === 0
          ? "Name cannot be empty"
          : parsedTime === null
            ? "Enter a valid time"
            : ""}
      </p>

      <div class="alarm-editor-actions">
        <button type="button" class="btn-confirm-cancel" onclick={cancelEditing}>
          Cancel
        </button>
        <button
          type="button"
          class="alarm-save-btn"
          disabled={!canSave}
          onclick={submitEdit}
        >
          Save
        </button>
      </div>
    </div>
  {/if}

  {#if confirmMode === "delete"}
    <div class="confirm-bar" role="region" aria-label="Confirm delete alarm">
      <span class="confirm-text">Delete "{alarm.name}"?</span>
      <div class="confirm-actions">
        <button type="button" class="btn-confirm-danger" onclick={confirmDelete}>
          Yes, Delete
        </button>
        <button type="button" class="btn-confirm-cancel" onclick={cancelConfirm}>
          Cancel
        </button>
      </div>
    </div>
  {/if}
</article>
